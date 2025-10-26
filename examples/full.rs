#[path = "utils/example.rs"]
mod example;

use bevy::asset::RenderAssetUsages;
use bevy::image::*;
use bevy::light::FogVolume;
use bevy::mesh::PlaneMeshBuilder;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy_feronia::prelude::*;
use example::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, RngCore, SeedableRng, rng};
use rand_pcg::Pcg64;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            strength: 0.2,
            micro_strength: 0.1,
            s_curve_strength: 0.1,
            bop_strength: 0.1,
            ..default()
        })
        .insert_resource(DensityMapConfig { size: 128 })
        /*   .insert_resource(ChunkDebugConfig {
            lod_colors: vec![
                RED_500.into(),
                ORANGE_500.into(),
                YELLOW_500.into(),
                WHITE.into(),
            ],
            aabb_color: GREEN_500.into(),
        })*/
        .add_plugins((
            ExamplePlugin,
            StandardScatterPlugin,
            InstancedWindAffectedScatterPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(Startup, (load_assets, setup_density_map))
        .add_systems(
            Update,
            check_assets_loaded.run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnEnter(AppState::InGame), spawn_scene)
        .add_systems(
            Update,
            (
                setup_density_map_inspection.run_if(resource_added::<DensityMap>),
                setup_height_map_inspection.run_if(resource_added::<HeightMapTexture>),
                scatter_on_keypress,
            ),
        )
        .add_observer(scatter_extended)
        .add_observer(scatter_instanced)
        .add_observer(update_density_map)
        .run()
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

#[derive(Resource)]
struct Scenes {
    landscape: Handle<Scene>,
    grass_lod_high: Handle<Scene>,
    grass_lod_medium: Handle<Scene>,
    grass_lod_low: Handle<Scene>,
    foliage_lod_high: Handle<Scene>,
    foliage_lod_medium: Handle<Scene>,
    foliage_lod_low: Handle<Scene>,
    trees_lod_high: Handle<Scene>,
    trees_lod_medium: Handle<Scene>,
    trees_lod_low: Handle<Scene>,
    rocks_lod_low: Handle<Scene>,
    rocks_lod_medium: Handle<Scene>,
    rocks_lod_high: Handle<Scene>,
    audio: Handle<AudioSource>,
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Scenes {
        landscape: asset_server.load("landscape_large.glb#Scene0"),
        grass_lod_high: asset_server.load("grass.glb#Scene0"),
        grass_lod_medium: asset_server.load("grass_medium_lod.glb#Scene0"),
        grass_lod_low: asset_server.load("grass_low_lod.glb#Scene0"),
        foliage_lod_high: asset_server.load("foliage_complex.glb#Scene0"),
        foliage_lod_medium: asset_server.load("foliage_complex_medium_lod.glb#Scene0"),
        foliage_lod_low: asset_server.load("foliage_complex_low_lod.glb#Scene0"),
        trees_lod_high: asset_server.load("trees_high_lod.glb#Scene0"),
        trees_lod_medium: asset_server.load("trees_medium_lod.glb#Scene0"),
        trees_lod_low: asset_server.load("trees_low_lod.glb#Scene0"),
        rocks_lod_low: asset_server.load("rocks_low_lod.glb#Scene0"),
        rocks_lod_medium: asset_server.load("rocks_medium_lod.glb#Scene0"),
        rocks_lod_high: asset_server.load("rocks_high_lod.glb#Scene0"),
        audio: asset_server
            .load("sounds/birds-singing-in-and-leaves-rustling-with-the-wind-14557.mp3"),
    });
}

fn setup_density_map_inspection(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    density_map: Res<DensityMap>,
) {
    cmd.spawn((
        Transform::from_xyz(10.0, 7.0, 5.0).looking_at(Vec3::new(0.0, 14.0, 1.0), Vec3::Y),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(1.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(density_map.0.clone()),
            unlit: true,
            ..default()
        })),
    ));
}

fn setup_height_map_inspection(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    height_map: Res<HeightMapTexture>,
) {
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

fn check_assets_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    handles: Res<Scenes>,
) {
    let all_loaded = [
        handles.landscape.id(),
        handles.grass_lod_high.id(),
        handles.grass_lod_medium.id(),
        handles.grass_lod_low.id(),
        handles.foliage_lod_high.id(),
        handles.foliage_lod_medium.id(),
        handles.foliage_lod_low.id(),
        handles.trees_lod_high.id(),
        handles.trees_lod_medium.id(),
        handles.trees_lod_low.id(),
        handles.rocks_lod_low.id(),
    ]
    .iter()
    .all(|id| {
        asset_server
            .get_load_state(*id)
            .is_some_and(|x| x.is_loaded())
    });

    if all_loaded {
        next_state.set(AppState::InGame);
    }
}

fn spawn_scene(
    mut cmd: Commands,
    density_map: Res<DensityMap>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
    mut ns_height_map: ResMut<NextState<HeightMapState>>,
    handles: Res<Scenes>,
    mut images: ResMut<Assets<Image>>,
) {
    let fog_texture = create_spherical_fog_texture(64);

    let fog_texture_handle = images.add(fog_texture);

    if let Some(image) = images.get_mut(&fog_texture_handle) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    }

    cmd.spawn((
        FogVolume {
            density_texture: Some(fog_texture_handle),
            density_factor: 0.05,
            fog_color: Color::WHITE,
            scattering_asymmetry: 0.6,
            ..default()
        },
        Transform::from_scale(Vec3::splat(300.).with_y(100.))
            .with_translation(Vec3::new(0., 15., 0.)),
    ));

    cmd.spawn((
        SceneRoot(handles.landscape.clone()),
        ScatterRoot::default(),
        MapHeight,
        ChunkRoot::default(),
        AudioPlayer::new(handles.audio.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        // Layers/Scattering should be ordered in respect to the ScatterOccupancyMap, i.e.,
        // later layers/scatters can't scatter on occupied areas from earlier layers/scatters.
        children![
            (
                Name::new("Rock Layer"),
                ScatterLayer::default(),
                ScatterLayerType::<StandardMaterial>::default(),
                DistributionDensity(10.0),
                InstanceRotationYaw::default(),
                InstanceScale { min: 1., max: 4. },
                InstanceJitter::default(),
                Avoidance(2.),
                children![
                    SceneRoot(handles.rocks_lod_high.clone()),
                    (
                        SceneRoot(handles.rocks_lod_medium.clone()),
                        LevelOfDetail(1)
                    ),
                    (SceneRoot(handles.rocks_lod_low.clone()), LevelOfDetail(2)),
                ]
            ),
            (
                bevy_feronia::extension::scatter::scatter_layer("Tree Layer"),
                DistributionDensity(8.0),
                InstanceRotationYaw::default(),
                InstanceScale { min: 4., max: 6. },
                InstanceJitter::default(),
                Avoidance(1.2),
                WindAffected,
                children![
                    SceneRoot(handles.trees_lod_high.clone()),
                    (
                        SceneRoot(handles.trees_lod_medium.clone()),
                        LevelOfDetail(1)
                    ),
                    (SceneRoot(handles.trees_lod_low.clone()), LevelOfDetail(2)),
                ]
            ),
            (
                bevy_feronia::extension::scatter::scatter_layer("Foliage Complex Layer"),
                DistributionDensity(20.0),
                InstanceRotationYaw::default(),
                InstanceScale { min: 8., max: 18. },
                InstanceJitter::default(),
                Avoidance(0.2),
                WindAffected,
                children![
                    // TODO figure out what's wrong with highest detail models
                    SceneRoot(handles.foliage_lod_medium.clone()),
                    (
                        SceneRoot(handles.foliage_lod_medium.clone()),
                        LevelOfDetail(1)
                    ),
                    (SceneRoot(handles.foliage_lod_low.clone()), LevelOfDetail(2))
                ]
            ),
            (
                bevy_feronia::instancing::scatter::scatter_layer("Instanced Grass Layer"),
                DistributionDensity(250.),
                DistributionPattern {
                    density_map: density_map.clone(),
                    scale: 1.0
                },
                InstanceJitter::default(),
                InstanceScale { min: 1., max: 1.5 },
                WindAffected,
                ScaleDensity,
                ScatterChunked,
                (
                    EnableBillboarding,
                    EdgeCorrectionFactor::default(),
                    CurveFactor::default(),
                    StrengthMultiplier(1.1),
                    MicroStrengthMultiplier(1.1),
                    SCurveStrength(0.2),
                    BopStrength(0.2)
                ),
                children![
                    SceneRoot(handles.grass_lod_high.clone()),
                    (
                        SceneRoot(handles.grass_lod_medium.clone()),
                        LevelOfDetail(1),
                    ),
                    (SceneRoot(handles.grass_lod_low.clone()), LevelOfDetail(2)),
                ],
            )
        ],
    ));

    ns_height_map.set(HeightMapState::Setup);
    ns_scatter.set(ScatterState::Setup);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_seed: ResMut<WorldSeed>,
    q_root: Single<Entity, With<ScatterRoot>>,
    mut mw_clear_root: MessageWriter<ClearScatterRoot>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    // Clean up all scattered instances.
    mw_clear_root.write((*q_root).into());

    // Generate a different world and update the density map.
    **world_seed = rng().next_u64();
    cmd.trigger(UpdateDensityMap);

    // Scatter the rocks.
    cmd.trigger(Scatter::<StandardMaterial>::new(*q_root));
}

fn scatter_extended(
    _: On<ScatterFinished<StandardMaterial>>,
    mut cmd: Commands,
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the foliage after the rocks.
    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*q_root));
}

fn scatter_instanced(
    _: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
    mut cmd: Commands,
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the grass last so it doesn't grow on occupied areas.
    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*q_root));
}

// TODO make expressive/descriptive configuration/plugin
#[derive(Resource)]
pub struct DensityMapConfig {
    pub size: u32,
}

#[derive(Resource, Deref, DerefMut)]
struct DensityMap(Handle<Image>);

#[derive(Event)]
struct UpdateDensityMap;

fn setup_density_map(mut commands: Commands) {
    commands.trigger(UpdateDensityMap);
}

/// Creates a density map with a Perlin noise base and empty spots stamped on top.
fn update_density_map(
    _: On<UpdateDensityMap>,
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<DensityMapConfig>,
    seed: Res<WorldSeed>,
) {
    let size = cfg.size;
    let mut data_buffer = vec![0; (size * size) as usize];
    let mut rng = Pcg64::seed_from_u64(**seed);

    let perlin = Perlin::new(**seed as u32);
    let sample_scale = 8.0;

    for y in 0..size {
        for x in 0..size {
            let point = [
                x as f64 / size as f64 * sample_scale,
                y as f64 / size as f64 * sample_scale,
            ];

            let noise_value = perlin.get(point);
            let byte_value = ((noise_value * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;

            data_buffer[(y * size + x) as usize] = byte_value;
        }
    }

    let num_spots = 100;
    let max_spot_radius = (size / 8) as f32;
    let min_spot_radius = (size / 40) as f32;

    // Empty spots
    for _ in 0..num_spots {
        let center_x = rng.random_range(0..size) as f32;
        let center_y = rng.random_range(0..size) as f32;

        let radius = rng.random_range(min_spot_radius..max_spot_radius);
        let radius_sq = radius * radius;

        // Bounds
        let x_min = (center_x - radius).floor().max(0.0) as u32;
        let x_max = (center_x + radius).ceil().min(size as f32) as u32;
        let y_min = (center_y - radius).floor().max(0.0) as u32;
        let y_max = (center_y + radius).ceil().min(size as f32) as u32;

        for y in y_min..y_max {
            for x in x_min..x_max {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < radius_sq {
                    // falloff
                    let t = dist_sq / radius_sq;
                    let intensity = (1.0 - t).clamp(0.0, 1.0);

                    let byte_value = ((1.0 - intensity) * 255.0) as u8;

                    let index = (y * size + x) as usize;
                    data_buffer[index] = data_buffer[index].min(byte_value);
                }
            }
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

/// Generates a 3D texture with a spherical density gradient.
///
/// The density is 1.0 (255) at the center and fades to 0.0 (0) at the edges.
fn create_spherical_fog_texture(size: u32) -> Image {
    let mut data = vec![0; (size * size * size) as usize];
    let center = size as f32 / 2.0;

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dz = z as f32 - center;

                let distance = (dx * dx + dy * dy + dz * dz).sqrt() / center;

                let density = (1.0 - distance).clamp(0.0, 1.0).powf(2.0);

                let value = (density * 255.0) as u8;

                let index = (x + y * size + z * size * size) as usize;
                data[index] = value;
            }
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: size,
        },
        TextureDimension::D3,
        data,
        TextureFormat::R8Unorm,
        RenderAssetUsages::default(),
    )
}
