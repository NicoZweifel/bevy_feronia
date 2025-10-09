#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_feronia::instancing::observers::instanced_scatter_observer;
use bevy_feronia::instancing::scatter::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use noise::{NoiseFn, Perlin};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .insert_resource(DensityMapConfig { size: 128 })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((ExamplePlugin, InstancedWindAffectedScatterPlugin))
        .insert_state(ScatterState::Setup)
        .insert_state(HeightMapState::Setup)
        .add_systems(Startup, (setup_density_map, setup).chain())
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>, density_map: Res<DensityMap>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        ChunkRoot::default(),
        ChunkRootSizeDim(4),
        ChunkLodConfig(vec![30.0.into(), f32::MAX.into()]),
        ChunkSizeScalarConfig(vec![1.into(), 2.into()]),
        MapHeight,
        LodConfig(vec![
            // Level 0: High
            30.0.into(),
            // Level 2: Low
            f32::MAX.into(),
        ]),
        children![(
            scatter_layer("Wind affected Grass Layer"),
            DistributionDensity(15.),
            DistributionPattern {
                density_map: density_map.clone(),
                scale: 1.
            },
            InstanceRotationYaw {
                min: 0.,
                max: std::f32::consts::PI * 2.
            },
            InstanceScale { min: 1., max: 1.5 },
            InstanceJitter(1.),
            WindAffected,
            ScaleDensity,
            ScatterChunked,
            EnableBillboarding,
            EdgeCorrectionFactor::default(),
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(1),
                )
            ]
        )],
    ))
    .observe(instanced_scatter_observer);
}

fn setup_density_map(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<DensityMapConfig>,
) {
    let size = config.size;
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
    commands.insert_resource(DensityMap(handle));
}

#[derive(Resource)]
pub struct DensityMapConfig {
    pub size: u32,
}

#[derive(Resource, Deref, DerefMut)]
struct DensityMap(Handle<Image>);
