#[path = "utils/example.rs"]
mod example;

use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::PlaneMeshBuilder;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_feronia::extension::observers::extended_scatter_observer;
use bevy_feronia::instancing::observers::instanced_scatter_observer;
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
            high_quality: true,
            lod_threshold: 10.0,
            ..default()
        })
        .insert_resource(DensityMapConfig { size: 128 })
        /*.insert_resource(ChunkDebugConfig {
            lod_colors: vec![
                RED_500.into(),
                ORANGE_500.into(),
                YELLOW_500.into(),
                WHITE.into(),
            ],
            aabb_color: GREEN_500.into(),
        })*/
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((
            ExamplePlugin,
            InstancedWindAffectedScatterPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .add_systems(Startup, (setup_density_map, setup).chain())
        .add_systems(
            Update,
            (
                setup_height_map_inspection.run_if(resource_added::<HeightMapTexture>),
                scatter_on_keypress,
            ),
        )
        .run()
}

fn setup_height_map_inspection(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMapTexture>,
) {
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

fn setup(mut cmd: Commands, assets: Res<AssetServer>, density_map: Res<DensityMap>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_large.glb#Scene0")),
        ScatterRoot::default(),
        MapHeight,
        ChunkRoot::default(),
        children![
            (
                bevy_feronia::instancing::scatter::scatter_layer("Instanced Grass Layer"),
                DistributionDensity(100.),
                DistributionPattern {
                    density_map: density_map.clone(),
                    scale: 1.0
                },
                InstanceRotationYaw {
                    min: 0.0,
                    max: std::f32::consts::PI * 2.0
                },
                InstanceJitter(1.0),
                InstanceScale { min: 2.0, max: 5.0 },
                WindAffected,
                children![
                    SceneRoot(assets.load("grass.glb#Scene0")),
                    (
                        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                        LevelOfDetail(1),
                    ),
                    (
                        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                        LevelOfDetail(2)
                    ),
                ],
            ),
            (
                bevy_feronia::extension::scatter::scatter_layer("Foliage Layer"),
                DistributionDensity(1.0),
                InstanceRotationYaw {
                    min: 0.0,
                    max: std::f32::consts::PI * 2.0
                },
                InstanceScale { min: 3., max: 10. },
                InstanceJitter(1.0),
                WindAffected,
                children![
                    (
                        LevelOfDetail(1),
                        SceneRoot(assets.load("foliage_complex_medium_lod.glb#Scene0")),
                    ),
                    (
                        LevelOfDetail(2),
                        SceneRoot(assets.load("foliage_complex_low_lod.glb#Scene0")),
                    )
                ]
            )
        ],
    ))
    .observe(instanced_scatter_observer)
    .observe(extended_scatter_observer);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Query<Entity, With<ScatterRoot>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    let targets = q_root.iter().collect::<Vec<_>>();

    cmd.trigger_targets(
        Scatter::<StandardMaterial, ExtendedWindAffectedMaterial>::new(),
        targets.clone(),
    );

    cmd.trigger_targets(
        Scatter::<StandardMaterial, InstancedWindAffectedMaterial>::new(),
        targets,
    );
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
