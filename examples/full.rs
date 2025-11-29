//! Showcases how to build a complex scene with quality settings by scattering different types
//! of assets (rocks, trees, foliage, and grass) onto a large landscape model.
//!
//! Ordered Scattering with:
//! * `StandardScatterPlugin` (for rocks)
//! * `ExtendedWindAffectedScatterPlugin` (for wind-affected trees and foliage)
//! * `InstancedWindAffectedScatterPlugin` (for high-density, wind-affected grass)
//!
//! The instanced grass layer is controlled by a procedurally generated `DensityMap`
//! (using Perlin noise) to create natural, patchy placement.
#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;
use bevy_feronia::asset::backend::scene_backend::SceneAssetBackendPlugin;
use bevy_feronia::prelude::*;
use bevy_feronia::quality::*;
use bevy_feronia::{extension, instancing};
use bevy_image::*;
use bevy_mesh::PlaneMeshBuilder;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use example::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, RngCore, SeedableRng, rng};
use rand_pcg::Pcg64;

fn main() -> AppExit {
    App::new()
        .insert_resource(ExamplePluginOptions {
            show_quality_settings: true,
            show_wind_settings: true,
            show_inspector: true,
        })
        .insert_resource(Wind { ..default() })
        .insert_resource(DensityMapConfig { size: 128 })
        /*
        .register_type::<ScatterAsset<StandardMaterial>>()
        .register_type::<ScatterAsset<ExtendedWindAffectedMaterial>>()
        .register_type::<ScatterAsset<InstancedWindAffectedMaterial>>()
         */
        .add_plugins((
            QualityPlugin,
            AssetSelectPlugin::<Scene>::new(),
            ExamplePlugin,
            SceneAssetBackendPlugin,
            StandardScatterPlugin,
            InstancedWindAffectedScatterPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(Startup, setup_density_map)
        .add_systems(
            Update,
            (
                load_assets.run_if(in_state(AppState::Setup)),
                check_assets_loaded.run_if(in_state(AppState::Loading)),
            ),
        )
        .add_systems(OnEnter(AppState::InGame), spawn_landscape)
        .add_systems(OnEnter(HeightMapState::Ready), spawn_scene)
        .add_systems(OnEnter(ScatterState::Ready), scatter)
        .add_systems(
            Update,
            (
                setup_density_map_inspection.run_if(resource_added::<DensityMap>),
                setup_height_map_inspection.run_if(resource_added::<HeightMapTexture>),
                scatter_on_keypress,
                respawn_scene
                    .run_if(in_state(AppState::InGame))
                    .run_if(resource_changed::<QualitySettings>)
                    .after(QualitySettingsUpdating),
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
    Setup,
    Loading,
    InGame,
}

#[derive(Resource)]
struct Scenes {
    // Always Loaded
    landscape: Handle<Scene>,
    audio: Handle<AudioSource>,

    // Low LODs are used by Low Quality (as close) and High Quality (as far)
    trees_lod_low: Handle<Scene>,
    trees_billboards: Handle<Scene>,

    foliage_lod_low: Handle<Scene>,

    grass_lod_low: Handle<Scene>,

    // Note: Low Quality settings use Med grass at LOD0
    grass_lod_medium: Handle<Scene>,

    rocks_lod_low: Handle<Scene>,

    // Conditionally Loaded (Quality Dependent)
    grass_lod_high: Handle<Scene>,

    foliage_lod_high: Handle<Scene>,
    foliage_lod_medium: Handle<Scene>,

    trees_lod_high: Handle<Scene>,
    trees_lod_medium: Handle<Scene>,

    rocks_lod_medium: Handle<Scene>,
    rocks_lod_high: Handle<Scene>,
}

impl Scenes {
    fn active_handles(&self) -> Vec<&Handle<Scene>> {
        [
            &self.landscape,
            &self.trees_lod_low,
            &self.trees_billboards,
            &self.foliage_lod_low,
            &self.grass_lod_low,
            &self.grass_lod_medium,
            &self.rocks_lod_low,
            &self.foliage_lod_medium,
            &self.trees_lod_medium,
            &self.rocks_lod_medium,
            &self.foliage_lod_high,
            &self.foliage_lod_medium,
            &self.trees_lod_high,
            &self.trees_lod_medium,
            &self.rocks_lod_medium,
            &self.rocks_lod_high,
        ]
        .into_iter()
        .filter(|x| x.id() != default::<Handle<Scene>>().id())
        .collect()
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    quality: Res<QualitySettings>,
    mut ns_app: ResMut<NextState<AppState>>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
    scenes: Option<ResMut<Scenes>>,
) {
    let load_opt =
        |existing: Option<Handle<Scene>>, condition: bool, path: &'static str| -> Handle<Scene> {
            existing
                .map(|h| {
                    asset_server
                        .get_load_state(h.id())
                        .is_some_and(|s| s.is_loaded())
                        .then(|| h)
                })
                .flatten()
                .or_else(|| condition.then(|| asset_server.load(path)))
                .unwrap_or_default()
        };

    // High/Ultra: Needs High, Medium, Low, Billboards
    // Medium:     Needs Medium, Low, Billboards (Usually excludes High, but see Grass below)
    // Low:        Needs Low, Billboards (excludes Medium, High)

    let is_high_tier = quality.model_quality == ModelQuality::High;
    let is_med_tier = quality.model_quality == ModelQuality::Medium || is_high_tier;

    // Grass uses `Except<IsLowQuality>` for High asset.
    let is_grass_high_needed = quality.model_quality != ModelQuality::Low;

    commands.insert_resource(Scenes {
        // Mandatory
        landscape: scenes.as_ref().map_or_else(
            || asset_server.load("landscape_large.glb#Scene0"),
            |s| s.landscape.clone(),
        ),
        audio: scenes.as_ref().map_or_else(
            || {
                asset_server
                    .load("sounds/birds-singing-in-and-leaves-rustling-with-the-wind-14557.mp3")
            },
            |s| s.audio.clone(),
        ),

        trees_lod_low: scenes.as_ref().map_or_else(
            || asset_server.load("trees.glb#Scene2"),
            |s| s.trees_lod_low.clone(),
        ),
        trees_billboards: scenes.as_ref().map_or_else(
            || asset_server.load("trees.glb#Scene0"),
            |s| s.trees_billboards.clone(),
        ),

        foliage_lod_low: scenes.as_ref().map_or_else(
            || asset_server.load("foliage_complex_low_lod.glb#Scene0"),
            |s| s.foliage_lod_low.clone(),
        ),

        grass_lod_low: scenes.as_ref().map_or_else(
            || asset_server.load("grass_low_lod.glb#Scene0"),
            |s| s.grass_lod_low.clone(),
        ),
        // Low Quality uses Med Grass at LOD0.
        grass_lod_medium: scenes.as_ref().map_or_else(
            || asset_server.load("grass_medium_lod.glb#Scene0"),
            |s| s.grass_lod_medium.clone(),
        ),

        rocks_lod_low: scenes.as_ref().map_or_else(
            || asset_server.load("rocks_low_lod.glb#Scene0"),
            |s| s.rocks_lod_low.clone(),
        ),

        // Conditional

        // Grass High is used by Medium and High settings
        grass_lod_high: load_opt(
            scenes.as_ref().map(|s| s.grass_lod_high.clone()),
            is_grass_high_needed,
            "grass.glb#Scene0",
        ),

        // Trees/Foliage/Rocks follow standard progression
        foliage_lod_high: load_opt(
            scenes.as_ref().map(|s| s.foliage_lod_high.clone()),
            is_high_tier,
            "foliage_complex.glb#Scene0",
        ),
        foliage_lod_medium: load_opt(
            scenes.as_ref().map(|s| s.foliage_lod_medium.clone()),
            is_med_tier,
            "foliage_complex_medium_lod.glb#Scene0",
        ),

        trees_lod_high: load_opt(
            scenes.as_ref().map(|s| s.trees_lod_high.clone()),
            is_high_tier,
            "trees.glb#Scene1",
        ),
        trees_lod_medium: load_opt(
            scenes.as_ref().map(|s| s.trees_lod_medium.clone()),
            is_med_tier,
            "trees.glb#Scene3",
        ),

        rocks_lod_high: load_opt(
            scenes.as_ref().map(|s| s.rocks_lod_high.clone()),
            is_high_tier,
            "rocks_high_lod.glb#Scene0",
        ),
        rocks_lod_medium: load_opt(
            scenes.as_ref().map(|s| s.rocks_lod_medium.clone()),
            is_med_tier,
            "rocks_medium_lod.glb#Scene0",
        ),
    });

    ns_app.set(AppState::Loading);
    ns_scatter.set(ScatterState::Loading);
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
    let all_loaded = handles.active_handles().iter().all(|h| {
        asset_server
            .get_load_state(h.id())
            .is_some_and(|s| s.is_loaded())
    });

    if all_loaded {
        next_state.set(AppState::InGame);
    }
}

fn scatter(mut cmd: Commands, root: Single<Entity, With<ScatterRoot>>) {
    cmd.trigger(Scatter::<StandardMaterial>::new(*root));
}

fn respawn_scene(
    mut cmd: Commands,
    root: Single<Entity, With<ScatterRoot>>,
    mut ns_app: ResMut<NextState<AppState>>,
) {
    info!("Quality settings changed, despawning and respawning scene...");

    cmd.entity(*root).despawn();

    ns_app.set(AppState::Setup);
}

#[derive(Component)]
pub struct Landscape;

fn spawn_landscape(
    mut cmd: Commands,
    mut ns_height_map: ResMut<NextState<HeightMapState>>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
    handles: Res<Scenes>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<QualitySettings>,
) {
    info!("Spawning scene with quality: {:?}", settings.quality);

    let fog_texture = create_spherical_fog_texture(64);
    let fog_texture_handle = images.add(fog_texture);

    if let Some(image) = images.get_mut(&fog_texture_handle) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    }

    /* too expensive and seems bugged TODO
    if settings.quality == Quality::Ultra || settings.quality == Quality::High {
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
    }
     */

    cmd.spawn((
        Name::new("Landscape"),
        Landscape,
        SceneRoot(handles.landscape.clone()),
        ScatterRoot::default(),
        LodConfig::from(settings.range_quality),
        MapHeight,
        ChunkRoot::default(),
        AudioPlayer::new(handles.audio.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
    ));

    ns_scatter.set(ScatterState::Setup);
    ns_height_map.set(HeightMapState::Setup);
}

fn spawn_scene(
    mut cmd: Commands,
    landscape: Single<Entity, With<Landscape>>,
    density_map: Res<DensityMap>,
    handles: Res<Scenes>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<QualitySettings>,
) {
    info!("Spawning scene with quality: {:?}", settings.quality);

    let fog_texture = create_spherical_fog_texture(64);
    let fog_texture_handle = images.add(fog_texture);

    if let Some(image) = images.get_mut(&fog_texture_handle) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    }

    /* too expensive and seems bugged TODO
    if settings.quality == Quality::Ultra || settings.quality == Quality::High {
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
    }
     */

    cmd.entity(*landscape).insert(children![
        (
            Name::new("Rock Layer"),
            ScatterLayer::default(),
            ScatterLayerType::<StandardMaterial>::default(),
            (
                DistributionDensity(15.),
                InstanceRotationYaw::default(),
                InstanceScale { min: 1., max: 2. },
                InstanceJitter::default(),
                Avoidance(2.5),
            ),
            children![
                (
                    AssetSelect::progressive(
                        handles.rocks_lod_high.clone(),
                        handles.rocks_lod_medium.clone(),
                        handles.rocks_lod_low.clone(),
                    ),
                    LevelOfDetail(0)
                ),
                (
                    AssetSelect::new(handles.rocks_lod_medium.clone())
                        .with_med(handles.rocks_lod_low.clone()),
                    LevelOfDetail(1)
                ),
                (
                    AssetSelect::new(handles.rocks_lod_low.clone()),
                    LevelOfDetail(2)
                )
            ]
        ),
        (
            extension::scatter_layer("Tree Layer"),
            (
                DistributionDensity(20.),
                InstanceRotationYaw::default(),
                InstanceScale { min: 1., max: 2. },
                InstanceJitter::default(),
                Avoidance(0.5),
            ),
            (
                AddIf::new(QualityRule::Sss, SssBundle::default()),
                AddIf::new(QualityRule::StaticShadows, StaticShadow::default()),
                AddIf::new(QualityRule::Wind, WindAffected::default()),
            ),
            children![
                (
                    AssetSelect::progressive(
                        handles.trees_lod_high.clone(),
                        handles.trees_lod_medium.clone(),
                        handles.trees_lod_low.clone(),
                    ),
                    LevelOfDetail(0),
                ),
                (
                    AssetSelect::progressive(
                        handles.trees_lod_medium.clone(),
                        handles.trees_lod_low.clone(),
                        handles.trees_billboards.clone()
                    ),
                    AddIf::new(QualityRule::LowModel, Unlit),
                    LevelOfDetail(1)
                ),
                (
                    AssetSelect::new(handles.trees_lod_low.clone())
                        .with_med(handles.trees_billboards.clone()),
                    AddIf::new(QualityRule::MediumModel, Unlit),
                    LevelOfDetail(2)
                ),
                (
                    AssetSelect::new(handles.trees_billboards.clone()),
                    Unlit,
                    LevelOfDetail(3)
                )
            ]
        ),
        (
            extension::scatter_layer("Foliage Complex Layer"),
            (
                DistributionDensity(20.),
                InstanceRotationYaw::default(),
                InstanceScale { min: 4., max: 8. },
                InstanceJitter::default(),
                Avoidance(0.2),
            ),
            (
                AddIf::new(
                    QualityRule::Sss,
                    (
                        SubsurfaceScattering,
                        SubsurfaceScatteringIntensity(4.),
                        SubsurfaceScatteringScale(5.),
                    )
                ),
                AddIf::new(QualityRule::StaticShadows, StaticShadow::default()),
                AddIf::new(QualityRule::Wind, WindAffected::default()),
            ),
            children![
                (
                    AssetSelect::progressive(
                        handles.foliage_lod_high.clone(),
                        handles.foliage_lod_medium.clone(),
                        handles.foliage_lod_low.clone(),
                    ),
                    LevelOfDetail(0)
                ),
                (
                    AssetSelect::new(handles.foliage_lod_medium.clone())
                        .with_med(handles.foliage_lod_low.clone()),
                    LevelOfDetail(1)
                ),
                (
                    AssetSelect::new(handles.foliage_lod_low.clone()),
                    LevelOfDetail(2)
                ),
            ]
        ),
        (
            instancing::scatter_layer("Instanced Grass Layer"),
            (
                DistributionDensity::from(settings.grass_density),
                DistributionPattern(density_map.clone()),
                InstanceJitter::default(),
                InstanceScale::default(),
                ScatterChunked,
                ScaleDensity,
                GpuCull,
            ),
            (
                EdgeCorrectionFactor::default(),
                CurveFactor::default(),
                Strength(1.2),
                MicroStrength(1.2),
                SCurveStrength(1.2),
                BopStrength(1.2),
                AnalyticalNormals,
                InstanceColor::new(Color::hsla(84., 0.49, 0.35, 1.), Color::BLACK),
                StaticBendStrength::default(),
                SpecularStrength(0.2),
                (
                    AddIf::new(QualityRule::DirectionalLights, DirectionalLights),
                    AddIf::new(QualityRule::PointLights, PointLights),
                    AddIf::new(QualityRule::Wind, WindAffected::default())
                ),
            ),
            children![
                (
                    AssetSelect::progressive(
                        handles.grass_lod_high.clone(),
                        handles.grass_lod_medium.clone(),
                        handles.grass_lod_low.clone(),
                    ),
                    LevelOfDetail(0)
                ),
                (
                    AssetSelect::progressive(
                        handles.grass_lod_high.clone(),
                        handles.grass_lod_medium.clone(),
                        handles.grass_lod_low.clone(),
                    ),
                    LevelOfDetail(1)
                ),
                (
                    AssetSelect::new(handles.grass_lod_medium.clone())
                        .with_med(handles.grass_lod_low.clone()),
                    LevelOfDetail(2)
                ),
            ],
        )
    ]);
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

    // Clean up all scattered instances.
    mw_clear_root.write((*root).into());

    // Generate a different world and update the density map.
    **world_seed = rng().next_u64();
    cmd.trigger(UpdateDensityMap);

    // Scatter the rocks.
    cmd.trigger(Scatter::<StandardMaterial>::new(*root));
}

fn scatter_extended(
    _: On<ScatterFinished<StandardMaterial>>,
    mut cmd: Commands,
    root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the foliage after the rocks.
    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
}

fn scatter_instanced(
    _: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
    mut cmd: Commands,
    root: Single<Entity, With<ScatterRoot>>,
) {
    // Scatter the grass last so it doesn't grow on occupied areas.
    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}

// TODO create density map plugin
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
///
/// Simulates an artist drawing density until tooling to do that is ready.
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
    let min_spot_radius = (size / 32) as f32;

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
