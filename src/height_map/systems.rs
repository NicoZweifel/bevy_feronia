use crate::height_map::state::HeightMapState;
use crate::prelude::*;
use bevy_asset::Assets;
use bevy_camera::{
    Camera, Camera3d, ImageRenderTarget, OrthographicProjection, Projection, RenderTarget,
    ScalingMode,
    primitives::Aabb,
    visibility::{NoFrustumCulling, RenderLayers},
};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::MeshMaterial3d;
use bevy_render::{
    render_resource::*,
    view::screenshot::{Screenshot, ScreenshotCaptured},
};
use bevy_state::state::NextState;
use bevy_transform::prelude::{GlobalTransform, Transform};
use bevy_utils::default;

use crate::core::utils::despawn;
#[cfg(feature = "trace")]
use tracing::{debug, info};

pub fn setup_config(
    mut cmd: Commands,
    q_pending_landscapes: Query<Entity, (With<MapHeight>, Without<Aabb>)>,
    q_processed_landscapes: Query<(&Aabb, &GlobalTransform), With<MapHeight>>,
) {
    if !q_pending_landscapes.is_empty() {
        return;
    }

    let mut min_pt = Vec3::splat(f32::MAX);
    let mut max_pt = Vec3::splat(f32::MIN);
    let mut found_any = false;

    for (aabb, transform) in &q_processed_landscapes {
        found_any = true;

        let matrix = transform.to_matrix();

        for i in 0..8 {
            let corner = aabb.min()
                + (aabb.max() - aabb.min())
                    * Vec3A::new((i & 1) as f32, ((i >> 1) & 1) as f32, ((i >> 2) & 1) as f32);
            let world_corner = matrix.transform_point3(corner.into());
            min_pt = min_pt.min(world_corner);
            max_pt = max_pt.max(world_corner);
        }
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
        render_layer: RenderLayers::layer(1),
    };

    println!("{config:?}");

    #[cfg(feature = "trace")]
    debug!(
        "HeightMapConfig ready: Center {:?}, Size {}",
        world_center, world_size
    );

    cmd.insert_resource(config);
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
    q_mesh: Query<(&Mesh3d, &GlobalTransform), Without<ScatterLayer>>,
    material: Res<HeightMapMaterialHandle>,
    cfg: Res<HeightMapConfig>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    let mut spawned_any = false;
    for landscape_root in &q_landscape {
        for child in q_children.iter_descendants(landscape_root) {
            let Ok((mesh, child_transform)) = q_mesh.get(child) else {
                continue;
            };

            cmd.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(material.0.clone()),
                Transform::from_matrix(child_transform.to_matrix()),
                cfg.render_layer.clone(),
                NoFrustumCulling,
                HeightMapGhost,
            ));
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

// bake_height_map REMAINS UNCHANGED
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

// setup_height_map_pipeline REMAINS UNCHANGED
pub fn setup_height_map_pipeline(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<HeightMapConfig>,
) {
    let world_size = cfg.world_size;
    let world_center = cfg.world_center;
    let map_resolution = 2048;

    let mut image =
        Image::new_target_texture(map_resolution, map_resolution, TextureFormat::R32Float);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let image_handle = images.add(image);

    cmd.spawn((
        HeightMapCamera,
        Camera {
            target: RenderTarget::Image(ImageRenderTarget::from(image_handle.clone())),
            order: -1,
            ..default()
        },
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
