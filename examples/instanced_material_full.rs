#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::PlaneMeshBuilder;
use bevy::prelude::*;
use bevy::render::primitives::MeshAabb;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_feronia::chunking::plugin::ChunkPlugin;
use bevy_feronia::instancing::observers::scatter_observer;
use bevy_feronia::prelude::*;
use example::*;
use noise::{NoiseFn, Perlin};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            strength: 0.8,
            micro_strength: 0.2,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            high_quality: false,
            lod_threshold: 10.0,
            ..default()
        })
        .insert_resource(DensityMapConfig { size: 128 })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into(), YELLOW_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            InstancedWindAffectedPlugin,
            ChunkPlugin,
            ScatterPlugin,
            HeightMapPlugin,
        ))
        .add_systems(Startup, (setup_density_map, setup).chain())
        .run()
}

fn setup(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMapTexture>,
    density_map: Res<DensityMap>,
) {
    cmd.spawn((
        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
        WindAffected,
    ));

    cmd.spawn((
        SceneRoot(assets.load("landscape_large.glb#Scene0")),
        ScatterRoot::default(),
        ChunkRoot::default(),
        MapHeight,
        children![(
            scatter_layer("Wind affected Grass Layer"),
            DistributionDensity(150.),
            DistributionPattern {
                density_map: density_map.clone(),
                scale: 1.0
            },
            InstanceRotationYaw {
                min: 0.0,
                max: std::f32::consts::PI * 2.0
            },
            InstanceScale { min: 1.0, max: 3.0 },
            InstanceJitter(0.1),
        )],
    ))
    .observe(
        scatter_observer::<
            WindAffectedTypes<InstancedWindAffectedMaterial>,
            WindAffectedType<InstancedWindAffectedMaterial>,
        >,
    );

    // Inspect the height map
    cmd.spawn((
        Transform::from_xyz(10.0, 5.0, 5.0).looking_at(Vec3::new(0.0, 14.0, 1.0), Vec3::Y),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(1.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(height_map.0.clone()),
            unlit: true,
            ..default()
        })),
    ));
}

fn setup_density_map(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<DensityMapConfig>,
) {
    let size = cfg.size;
    let mut data_buffer = vec![0; (size * size) as usize];
    let perlin = Perlin::new(1);
    let sample_scale = 8.0;

    for y in 0..size {
        for x in 0..size {
            let point = [x as f64 / size as f64, y as f64 / size as f64];
            let noise_value = perlin.get([point[0] * sample_scale, point[1] * sample_scale]);
            let byte_value = ((noise_value * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            data_buffer[(y * size + x) as usize] = byte_value;
        }
    }

    let mut density_image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data_buffer,
        TextureFormat::R8Unorm,
        default(),
    );

    density_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("Foliage Density Sampler".into()),
        address_mode_u: ImageAddressMode::MirrorRepeat,
        address_mode_v: ImageAddressMode::MirrorRepeat,
        ..default()
    });

    let handle = images.add(density_image);
    cmd.insert_resource(DensityMap(handle));
}

#[derive(Resource)]
pub struct DensityMapConfig {
    pub size: u32,
}

#[derive(Resource, Deref, DerefMut)]
struct DensityMap(Handle<Image>);
