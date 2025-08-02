#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_feronia::chunking::plugin::ChunkPlugin;
use bevy_feronia::extension::observers::scatter_observer;
use bevy_feronia::height_map::systems::setup_height_map_pipeline;
use bevy_feronia::prelude::*;
use example::*;
use noise::{NoiseFn, Perlin};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            strength: 0.6,
            micro_strength: 0.25,
            ..default()
        })
        .insert_resource(DensityMapConfig { size: 128 })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into(), YELLOW_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            ExtendedWindAffectedPlugin,
            ChunkPlugin,
            HeightMapPlugin,
            ScatterPlugin,
        ))
        .add_systems(
            Startup,
            (setup_density_map, setup.after(setup_height_map_pipeline)).chain(),
        )
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>, density_map: Res<DensityMap>) {
    cmd.spawn((
        SceneRoot(assets.load("foliage_complex.glb#Scene0")),
        WindAffected,
    ));

    cmd.spawn((
        SceneRoot(assets.load("landscape_large.glb#Scene0")),
        ScatterRoot::default(),
        MapHeight,
        ChunkRoot::default(),
        children![(
            scatter_layer("Wind affected Foliage Layer"),
            DistributionDensity(1.),
            DistributionPattern {
                density_map: density_map.clone(),
                scale: 1.0
            },
            InstanceRotationYaw {
                min: 0.0,
                max: std::f32::consts::PI * 2.0
            },
            InstanceScale { min: 1.0, max: 3.0 },
        )],
    ))
    .observe(scatter_observer);
}

fn scatter_on_keypress(
    prototypes: Res<WindAffectedTypes<ExtendedWindAffectedMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Query<Entity, With<ScatterRoot>>,
    mut ew_scatter: EventWriter<Scatter>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    println!("Scattering plants...");

    for e in &q_root {
        ew_scatter.write(Scatter(e));
    }
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
