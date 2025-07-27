use crate::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::view::NoFrustumCulling;

pub fn setup_materials(mut cmd: Commands, mut materials: ResMut<Assets<HeightMapMaterial>>) {
    let handle = materials.add(HeightMapMaterial {});
    cmd.insert_resource(HeightMapMaterialHandle(handle));
}

pub fn create_height_map_ghost(
    mut cmd: Commands,
    q_landscape: Query<Entity, (With<Landscape>, Without<HeightMapped>)>,
    q_children: Query<&Children>,
    q_mesh: Query<(&Mesh3d, &GlobalTransform)>,
    material: Res<HeightMapMaterialHandle>,
    cfg: Res<HeightMapConfig>,
) {
    for landscape_root in &q_landscape {
        for child in q_children.iter_descendants(landscape_root) {
            if let Ok((mesh, transform)) = q_mesh.get(child) {
                cmd.spawn((
                    Mesh3d(mesh.0.clone()),
                    MeshMaterial3d(material.0.clone()),
                    transform.compute_transform(),
                    cfg.render_layer.clone(),
                    NoFrustumCulling,
                ));

                cmd.entity(landscape_root).insert(HeightMapped);

                info!("Spawned ghost landscape for height map generation.");
            }
        }
    }
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

    image.texture_descriptor.usage = image.texture_descriptor.usage | TextureUsages::COPY_SRC;

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
