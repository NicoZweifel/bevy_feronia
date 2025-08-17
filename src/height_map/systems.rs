use crate::height_map::state::HeightMapState;
use crate::prelude::*;
use crate::scatter::utils::combine_aabbs;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

pub fn setup_materials(
    mut cmd: Commands,
    mut materials: ResMut<Assets<HeightMapMaterial>>,
    cfg: Res<HeightMapConfig>,
) {
    let handle = materials.add(HeightMapMaterial::from(cfg.into_inner()));

    cmd.insert_resource(HeightMapMaterialHandle(handle));
}

pub fn setup_config(
    mut cmd: Commands,
    q_pending_landscapes: Query<Entity, (With<MapHeight>, Without<Aabb>)>,
    q_processed_landscapes: Query<&Aabb, With<MapHeight>>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    if !q_pending_landscapes.is_empty() {
        return;
    }

    if q_processed_landscapes.is_empty() {
        next_state.set(HeightMapState::Loading);
        return;
    }

    let mut total_aabb: Option<Aabb> = None;

    for aabb in &q_processed_landscapes {
        if let Some(total) = &mut total_aabb {
            *total = combine_aabbs(total, aabb);
        } else {
            total_aabb = Some(*aabb);
        }
    }

    let Some(aabb) = total_aabb else {
        return;
    };

    let size_x = aabb.max().x - aabb.min().x;
    let size_z = aabb.max().z - aabb.min().z;
    let world_size = size_x.max(size_z);
    let config = HeightMapConfig {
        world_size,
        world_height_range: aabb.min().y..aabb.max().y,
        render_layer: RenderLayers::layer(1),
    };

    debug!("HeightMapConfig created from root AABB:");
    debug!("   - World Size: {:.2}", config.world_size);
    debug!("   - Min Height: {:.2}", config.world_height_range.start);
    debug!("   - Max Height: {:.2}", config.world_height_range.end);

    cmd.insert_resource(config);
}

pub fn skip_setup(
    q_landscapes: Query<Entity, With<MapHeight>>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    if !q_landscapes.is_empty() {
        return;
    };

    info!("Skipping HeightMap setup");

    next_state.set(HeightMapState::Ready);
}

pub fn finish_setup(mut next_state: ResMut<NextState<HeightMapState>>) {
    next_state.set(HeightMapState::Loading);
}

pub fn create_height_map_ghost(
    mut cmd: Commands,
    q_landscape: Query<Entity, (With<MapHeight>, Without<HeightMapped>)>,
    q_children: Query<&Children>,
    q_mesh: Query<(&Mesh3d, &GlobalTransform)>,
    material: Res<HeightMapMaterialHandle>,
    cfg: Res<HeightMapConfig>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    for landscape_root in &q_landscape {
        for child in q_children.iter_descendants(landscape_root) {
            let Ok((mesh, transform)) = q_mesh.get(child) else {
                continue;
            };

            cmd.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(material.0.clone()),
                transform.compute_transform(),
                cfg.render_layer.clone(),
                NoFrustumCulling,
            ));

            cmd.entity(landscape_root).insert(HeightMapped);

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
    // Wait a few frames to bake
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

                println!("capture");
                next_state.set(HeightMapState::Ready);
            },
        );
}

pub fn setup_height_map_pipeline(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<HeightMapConfig>,
) {
    let world_size = cfg.world_size;

    let map_resolution = 2048;

    let mut image =
        Image::new_target_texture(map_resolution, map_resolution, TextureFormat::R32Float);

    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;

    let image_handle = images.add(image);

    cmd.spawn((
        Camera {
            target: RenderTarget::Image(ImageRenderTarget::from(image_handle.clone())),
            order: -1,
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(0.0, world_size / 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        Projection::from(OrthographicProjection {
            area: Rect::new(
                -world_size / 2.0,
                -world_size / 2.0,
                world_size / 2.0,
                world_size / 2.0,
            ),
            // Ensure the clipping planes encompass the entire landscape's height.
            near: 0.1,
            far: world_size,
            scale: 1.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: world_size,
                height: world_size,
            },
        }),
        cfg.render_layer.clone(),
    ));

    cmd.insert_resource(HeightMapTexture(image_handle));
}
