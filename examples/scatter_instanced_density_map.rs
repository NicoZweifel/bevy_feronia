#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::asset::backend::mesh_material_backend::MeshMaterialAssetBackendPlugin;
use bevy_feronia::instancing::scatter_layer;
use bevy_feronia::prelude::*;
use bevy_image::*;
use bevy_render::render_resource::*;
use example::*;
use noise::{NoiseFn, Perlin};
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(GlobalWind::from(WindPreset::Mild))
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            ..default()
        })
        .insert_resource(DensityMapConfig { size: 128 })
        .add_plugins((
            ExamplePlugin,
            // This example spawns everything in startup, so we can just use the MeshMaterialAssetBackendPlugin
            MeshMaterialAssetBackendPlugin,
            InstancedWindAffectedScatterPlugin,
        ))
        .insert_state(HeightMapState::Setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Startup, (setup_density_map, setup).chain())
        .add_systems(
            Update,
            (
                scatter_on_keypress.in_set(ScatterSet::Ready),
                select_map_on_keypress,
            ),
        )
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>, density_map: Res<DensityMap>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Wind affected Grass Layer"),
            // Scatter options
            (
                DistributionDensity(100.),
                DistributionPattern(density_map.images[0].clone()),
                InstanceRotationYaw::default(),
                InstanceScale::default(),
                ScaleDensity,
            ),
            // Material options
            (
                WindAffected,
                EdgeCorrectionFactor::default(),
                AnalyticalNormals,
                DirectionalLights,
                PointLights,
            ),
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_medium_lod.glb#Scene0")),
                    LevelOfDetail(1),
                ),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(2),
                )
            ]
        )],
    ));
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_seed: ResMut<WorldSeed>,
    root: Single<Entity, With<ScatterRoot>>,
    mut mw_clear_root: MessageWriter<ClearScatterRoot>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    mw_clear_root.write((*root).into());
    **world_seed = rng().next_u64();

    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}

fn select_map_on_keypress(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    density_map: Res<DensityMap>,
    mut distribution_pattern_query: Query<&mut DistributionPattern>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        for mut pattern in &mut distribution_pattern_query {
            pattern.0 = density_map.images[0].clone();
        }
        info!("Density map is set to: Generated perlin noise");
    };

    if keyboard_input.just_pressed(KeyCode::Digit2) {
        for mut pattern in &mut distribution_pattern_query {
            pattern.0 = density_map.images[1].clone();
        }
        info!("Density map is set to: 'assets/density_map.png' image file");
    };
}

/// Simulates an artist drawing density until tooling to do that is ready.
fn setup_density_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
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

    let images = [images.add(density_image), assets.load("density_map.png")];

    commands.insert_resource(DensityMap { images });
}

#[derive(Resource)]
pub struct DensityMapConfig {
    pub size: u32,
}

#[derive(Resource)]
struct DensityMap {
    images: [Handle<Image>; 2],
}
