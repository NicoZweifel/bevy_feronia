use crate::core::Sampler;
use crate::height_map::state::HeightMapState;
use crate::prelude::*;

use bevy_asset::Assets;
use bevy_camera::{
    Camera, Camera3d, ImageRenderTarget, OrthographicProjection, Projection, RenderTarget,
    ScalingMode,
    primitives::Aabb,
    visibility::NoFrustumCulling,
};
use bevy_ecs::prelude::*;
use bevy_gizmos::gizmos::Gizmos;
use bevy_image::Image;
use bevy_light::{NotShadowCaster, NotShadowReceiver};
use bevy_math::*;
use bevy_mesh::Mesh3d;
use bevy_mesh::skinning::SkinnedMesh;
use bevy_pbr::MeshMaterial3d;
use bevy_render::batching::NoAutomaticBatching;
use bevy_render::{
    render_resource::*,
    view::screenshot::{Screenshot, ScreenshotCaptured},
};
use bevy_state::state::NextState;
use bevy_transform::prelude::{GlobalTransform, Transform};
use bevy_utils::default;

use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::utils::despawn;

#[cfg(feature = "trace")]
use tracing::{debug, info};

pub fn setup_config(
    mut cmd: Commands,
    q_pending_landscapes: Query<Entity, (With<MapHeight>, Without<Aabb>)>,
    q_processed_landscapes: Query<&Aabb, With<MapHeight>>,
) {
    if !q_pending_landscapes.is_empty() {
        return;
    }

    let mut min_pt = Vec3::splat(f32::MAX);
    let mut max_pt = Vec3::splat(f32::MIN);
    let mut found_any = false;

    for aabb in &q_processed_landscapes {
        found_any = true;
        min_pt = min_pt.min(aabb.min().into());
        max_pt = max_pt.max(aabb.max().into());
    }

    if !found_any {
        return;
    }

    let size = max_pt - min_pt;
    let world_size = size.x.max(size.z);

    let center_3d = min_pt + size * 0.5;
    let world_center = Vec2::new(center_3d.x, center_3d.z);

    let config = HeightMapConfig {
        world_size,
        world_center,
        world_height_range: min_pt.y..max_pt.y,
        ..default()
    };

    cmd.insert_resource(config);

    #[cfg(feature = "trace")]
    debug!(
        "HeightMapConfig: Center {:?}, Size {}",
        world_center, world_size
    );
}

pub fn setup_materials(
    mut cmd: Commands,
    mut materials: ResMut<Assets<HeightMapMaterial>>,
    cfg: Res<HeightMapConfig>,
) {
    let handle = materials.add(HeightMapMaterial::from(cfg.into_inner()));
    cmd.insert_resource(HeightMapMaterialHandle(handle));
}

pub fn skip_setup(
    q_landscapes: Query<Entity, With<MapHeight>>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    if !q_landscapes.is_empty() {
        return;
    };
    #[cfg(feature = "trace")]
    info!("Skipping HeightMap setup");
    next_state.set(HeightMapState::Ready);
}

pub fn finish_setup(mut next_state: ResMut<NextState<HeightMapState>>) {
    next_state.set(HeightMapState::Ghost);
}

pub fn create_height_map_ghost(
    mut cmd: Commands,
    q_landscape: Query<Entity, (With<MapHeight>, Without<HeightMapped>)>,
    q_children: Query<&Children, Without<ScatterLayer>>,
    q_mesh: Query<(&Mesh3d, &GlobalTransform, Option<&SkinnedMesh>), Without<ScatterLayer>>,
    material: Res<HeightMapMaterialHandle>,
    cfg: Res<HeightMapConfig>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    let mut spawned_any = false;

    for landscape_root in &q_landscape {
        for (mesh_handle, child_transform, skinned_mesh) in q_children
            .iter_descendants(landscape_root)
            .filter_map(|child| q_mesh.get(child).ok())
        {
            let mut ghost = cmd.spawn((
                Mesh3d(mesh_handle.0.clone()),
                MeshMaterial3d(material.0.clone()),
                Transform::from_matrix(child_transform.to_matrix()),
                cfg.render_layer.clone(),
                NoFrustumCulling,
                HeightMapGhost,
                NotShadowCaster,
                NotShadowReceiver,
                NoAutomaticBatching,
            ));

            if let Some(skinned) = skinned_mesh {
                ghost.insert(skinned.clone());
            }
            spawned_any = true;
        }

        if spawned_any {
            cmd.entity(landscape_root).insert(HeightMapped);
            #[cfg(feature = "trace")]
            debug!("HeightMapGhost created");
            next_state.set(HeightMapState::Baking);
        }
    }
}

pub fn bake_height_map(
    mut commands: Commands,
    height_map_texture: Res<HeightMapTexture>,
    mut next_state: ResMut<NextState<HeightMapState>>,
    mut counter: Local<u32>,
) {
    if *counter < 100 {
        *counter += 1;
        return;
    }

    next_state.set(HeightMapState::Saving);

    commands
        .spawn(Screenshot::image(height_map_texture.0.clone()))
        .observe(
            |trigger: On<ScreenshotCaptured>,
             mut images: ResMut<Assets<Image>>,
             mut cmd: Commands,
             mut next_state: ResMut<NextState<HeightMapState>>| {
                let mut image = trigger.clone();
                image.asset_usage = default();
                cmd.insert_resource(HeightMap(images.add(image)));
                next_state.set(HeightMapState::Ready);
            },
        );
}

pub fn setup_height_map_pipeline(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<HeightMapConfig>,
    // TODO, should work on a component basis on multiple roots with multiple landscapes.
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    let world_size = cfg.world_size;
    let world_center = cfg.world_center;
    let map_resolution = 2048;

    let mut image = Image::new_target_texture(
        map_resolution,
        map_resolution,
        TextureFormat::R32Float,
        None,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let image_handle = images.add(image);

    cmd.spawn((
        HeightMapCamera,
        Camera {
            order: -1,
            ..default()
        },
        RenderTarget::Image(ImageRenderTarget::from(image_handle.clone())),
        Camera3d::default(),
        Transform::from_xyz(
            world_center.x,
            cfg.world_height_range.end + 50.0,
            world_center.y,
        )
        .looking_at(Vec3::new(world_center.x, 0.0, world_center.y), Vec3::NEG_Z),
        Projection::from(OrthographicProjection {
            area: Rect::new(
                -world_size / 2.0,
                -world_size / 2.0,
                world_size / 2.0,
                world_size / 2.0,
            ),
            near: 0.1,
            far: (cfg.world_height_range.end - cfg.world_height_range.start) + 100.0,
            scale: 1.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: ScalingMode::Fixed {
                width: world_size,
                height: world_size,
            },
        }),
        cfg.render_layer.clone(),
        ChildOf(*q_root),
    ));

    cmd.insert_resource(HeightMapTexture(image_handle));
}

pub fn teardown_height_map_pipeline(
    mut cmd: Commands,
    q_ghosts: Query<Entity, With<HeightMapGhost>>,
    q_camera: Query<Entity, With<HeightMapCamera>>,
    q_mapped_landscapes: Query<Entity, With<HeightMapped>>,
) {
    despawn(&mut cmd, q_ghosts.iter().chain(q_camera));

    cmd.remove_resource::<HeightMapTexture>();
    cmd.remove_resource::<HeightMapMaterialHandle>();

    for entity in &q_mapped_landscapes {
        cmd.entity(entity).remove::<HeightMapped>();
    }
}

pub fn draw_height_map(
    mut gizmos: Gizmos,
    images: Res<Assets<Image>>,
    config: Res<HeightMapConfig>,
    debug_config: Res<HeightMapDebugConfig>,
    height_map: Res<HeightMap>,
    q_root: Query<&GlobalTransform, With<ScatterRoot>>,
) {
    let Some(image) = images.get(&height_map.0) else {
        return;
    };

    let sampler = HeightMapCpuSampler::new(image, &config);

    let range = (config.world_size / 2.) as i32;

    for gtf in &q_root {
        for x in -range..range {
            for z in -range..range {
                let local_x = x as f32;
                let local_z = z as f32;

                let height = sampler.sample(Vec3::new(local_x, 0.0, local_z));
                let start = gtf.transform_point(Vec3::new(local_x, height, local_z));

                gizmos.line(start, start.with_y(start.y + 0.5), *debug_config);
            }
        }
    }
}
