#[path = "camera_controller.rs"]
mod camera_controller;

#[path = "quality.rs"]
pub mod quality;

use quality::*;

#[cfg(not(feature = "dlss"))]
use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::diagnostic::*;
use bevy::light::ShadowFilteringMethod;
use bevy::post_process::bloom::Bloom;
#[cfg(all(feature = "dlss"))]
use bevy::{
    anti_alias::dlss::{Dlss, DlssPerfQualityMode, DlssProjectId},
    asset::uuid,
};
use bevy::{
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    light::VolumetricLight,
    prelude::*,
    render::view::ColorGrading,
};
use bevy_feronia::prelude::*;
use bevy_image::{ImageSampler, ImageSamplerDescriptor};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};
use bevy_light::{CascadeShadowConfig, DirectionalLightShadowMap};
use bevy_render::view::Hdr;
use bevy_show_prepass::{ShowPrepass, ShowPrepassPlugin};
use camera_controller::*;
use iyes_perf_ui::prelude::*;

#[derive(Resource, Default, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ExamplePluginOptions {
    pub show_quality_settings: bool,
    pub show_wind_settings: bool,
    pub show_inspector: bool,
    pub show_debug_options: bool,
}

#[derive(Resource, Default, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ExampleDebugOptions {
    pub debug_chunks: bool,
    pub debug_height_map: bool,
    pub debug_occupancy_map: bool,
    pub debug_scattered_entities: bool,
}

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(feature = "dlss"))]
        // NOTE: This is an example project ID, you should generate your own uuid.
        app.insert_resource(DlssProjectId(uuid::uuid!(
            "edac5c37-87f0-4e5c-be93-3636dd13677a"
        )));

        app.init_resource::<ExamplePluginOptions>()
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
                PerfUiPlugin,
                ShowPrepassPlugin,
            ))
            .add_plugins((
                EguiPlugin::default(),
                WorldInspectorPlugin::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_inspector),
                ResourceInspectorPlugin::<GlobalWind>::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_wind_settings),
                ResourceInspectorPlugin::<QualitySettings>::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_quality_settings),
                ResourceInspectorPlugin::<ExampleDebugOptions>::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_debug_options),
                ResourceInspectorPlugin::<ChunkDebugConfig>::default().run_if(
                    resource_exists::<ChunkDebugConfig>
                        .and(|res: Res<ExampleDebugOptions>| res.debug_chunks),
                ),
                ResourceInspectorPlugin::<HeightMapDebugConfig>::default().run_if(
                    resource_exists::<HeightMapDebugConfig>
                        .and(|res: Res<ExampleDebugOptions>| res.debug_height_map),
                ),
                ResourceInspectorPlugin::<ScatterOccupancyMapDebugConfig>::default().run_if(
                    resource_exists::<ScatterOccupancyMapDebugConfig>
                        .and(|res: Res<ExampleDebugOptions>| res.debug_occupancy_map),
                ),
            ))
            .add_systems(
                Update,
                (|mut cmd: Commands,
                  res: Res<ExamplePluginOptions>,
                  debug_options: Option<Res<ExampleDebugOptions>>| {
                    if res.show_debug_options && debug_options.is_none() {
                        cmd.init_resource::<ExampleDebugOptions>();
                    } else if !res.show_debug_options && debug_options.is_some() {
                        cmd.remove_resource::<ExampleDebugOptions>();
                    }
                })
                    .run_if(resource_exists::<ExamplePluginOptions>),
            )
            .add_systems(
                Update,
                (
                    |mut cmd: Commands,
                     res: Res<ExampleDebugOptions>,
                     chunk_debug_config: Option<Res<ChunkDebugConfig>>,
                     height_debug_config: Option<Res<HeightMapDebugConfig>>,
                     occupancy_debug_config: Option<Res<ScatterOccupancyMapDebugConfig>>| {
                        if chunk_debug_config.is_none() && res.debug_chunks {
                            cmd.init_resource::<ChunkDebugConfig>();
                        } else if chunk_debug_config.is_some() && !res.debug_chunks {
                            cmd.remove_resource::<ChunkDebugConfig>();
                        }

                        if height_debug_config.is_none() && res.debug_height_map {
                            cmd.init_resource::<HeightMapDebugConfig>();
                        } else if height_debug_config.is_some() && !res.debug_height_map {
                            cmd.remove_resource::<HeightMapDebugConfig>();
                        }

                        if occupancy_debug_config.is_none() && res.debug_occupancy_map {
                            cmd.init_resource::<ScatterOccupancyMapDebugConfig>();
                        } else if occupancy_debug_config.is_some() && !res.debug_occupancy_map {
                            cmd.remove_resource::<ScatterOccupancyMapDebugConfig>();
                        }
                    },
                )
                    .run_if(resource_exists::<ExampleDebugOptions>),
            )
            .add_plugins(CameraControllerPlugin)
            .add_systems(
                Startup,
                (setup, spawn_directional_light.in_set(QualitySettingsSetup)),
            )
            .add_systems(
                Update,
                (
                    (
                        update_extended_materials.run_if(resource_exists::<Assets<ExtendedWindAffectedMaterial>>),
                        update_instanced_materials.run_if(resource_exists::<Assets<InstancedWindAffectedMaterial>>)
                    ).run_if(resource_exists_and_changed::<ExampleDebugOptions>),
                    setup_camera,
                    anisotropic_filtering,
                    (rotate_sun, move_on_key_press, choose_show_prepass_mode).in_set(ScatterSet::Ready),
                    respawn_directional_light
                        .run_if(resource_changed::<DirectionalLightShadowMap>)
                        .in_set(QualitySettingsUpdated),
                ),
            );
    }
}

fn update_extended_materials(
    mut assets: ResMut<Assets<ExtendedWindAffectedMaterial>>,
    res: Res<ExampleDebugOptions>,
) {
    for (_, asset) in assets.iter_mut() {
        asset.extension.options.debug = res.debug_scattered_entities;
    }
}

fn update_instanced_materials(
    mut assets: ResMut<Assets<InstancedWindAffectedMaterial>>,
    res: Res<ExampleDebugOptions>,
) {
    for (_, asset) in assets.iter_mut() {
        asset.options.debug = res.debug_scattered_entities;
    }
}

fn respawn_directional_light(
    mut cmd: Commands,
    light: Single<
        (Entity, &DirectionalLight, &CascadeShadowConfig, &Transform),
        With<DirectionalLight>,
    >,
) {
    let (entity, light, cfg, tf) = light.into_inner();

    cmd.entity(entity).despawn();

    cmd.spawn((
        light.clone(),
        cfg.clone(),
        tf.clone(),
        VolumetricLight,
        ShadowFilteringMethod::Temporal,
    ));
}

fn spawn_directional_light(mut cmd: Commands) {
    cmd.spawn((
        DirectionalLight {
            // NOTE: Direct sunlight has over-exposure with the SkyBox ambient
            // FULL_DAYLIGHT seems a bit low but 30_000. seems fine.
            illuminance: 30_000.,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.95),
            ..default()
        },
        VolumetricLight,
        Transform::from_xyz(2., 2., 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ShadowFilteringMethod::Temporal,
    ));
}

pub fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn((
        Mesh3d(meshes.add(Sphere::new(3.0).mesh().uv(120, 64))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 5.0, 0.5),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0., 5., 0.),
    ))
    .with_child(PointLight {
        radius: 3.0,
        color: Color::srgb(0.1, 0.1, 1.),
        shadows_enabled: false,
        range: 20.,
        intensity: 500_000.,
        ..default()
    });

    cmd.spawn(PerfUiDefaultEntries::default());
}

pub fn setup_camera(mut cmd: Commands, asset_server: Res<AssetServer>, q_camera: Query<&Camera>) {
    if !q_camera.is_empty() {
        return;
    };

    cmd.spawn((
        Camera::default(),
        Hdr,
        Controller::default(),
        Camera3d::default(),
        ColorGrading::default(),
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(-30., 20., 30.).looking_at(Vec3::ZERO, Vec3::Y),
        Center,
        Skybox {
            image: asset_server.load("skybox.ktx2"),
            brightness: 10000.,
            ..default()
        },
        ShadowFilteringMethod::Temporal,
        #[cfg(all(feature = "dlss"))]
        (
            Msaa::Off,
            Dlss {
                perf_quality_mode: DlssPerfQualityMode::Dlaa,
                ..default()
            },
        ),
        #[cfg(not(feature = "dlss"))]
        (Msaa::Off, TemporalAntiAliasing::default()),
        bevy_pbr::ScreenSpaceAmbientOcclusion::default(),
        bevy::light::VolumetricFog {
            ambient_intensity: 0.1,
            ..default()
        },
    ));
}

fn anisotropic_filtering(
    mut mv_asset: MessageReader<AssetEvent<Image>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for ev in mv_asset.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else {
            continue;
        };

        let Some(image) = image_assets.get_mut(*id) else {
            continue;
        };

        image.sampler = match &image.sampler {
            ImageSampler::Default => ImageSampler::Descriptor(ImageSamplerDescriptor {
                anisotropy_clamp: 16,
                ..ImageSamplerDescriptor::linear()
            }),
            ImageSampler::Descriptor(image_sampler_descriptor) => {
                ImageSampler::Descriptor(ImageSamplerDescriptor {
                    anisotropy_clamp: 16,
                    ..image_sampler_descriptor.clone()
                })
            }
        };
    }
}

const SUN_ROTATION_SPEED: f32 = 0.5;

fn rotate_sun(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut sun_query: Query<&mut Transform, With<DirectionalLight>>,
    mut sky_query: Query<&mut Skybox>,
) {
    let mut rotation_direction = 0.0;
    if keys.pressed(KeyCode::KeyQ) {
        rotation_direction += 1.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        rotation_direction -= 1.0;
    }

    if rotation_direction != 0.0 {
        let rotation_amount = rotation_direction * SUN_ROTATION_SPEED * time.delta_secs();
        let rotation = Quat::from_rotation_y(rotation_amount);

        for mut transform in &mut sun_query {
            transform.rotate_around(Vec3::ZERO, rotation);
        }

        for mut transform in &mut sky_query {
            transform.rotation = rotation * transform.rotation;
        }
    }
}

fn move_on_key_press(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<ScatterRoot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard_input.pressed(KeyCode::KeyM) {
        return;
    }

    let delta = time.delta_secs();
    let rotation = Quat::from_rotation_y(0.5 * delta);

    let step = time.elapsed_secs().cos() * 10.0 * delta;

    for mut transform in &mut query {
        transform.translation.y += step;

        transform.rotate_around(Vec3::ZERO, rotation);
    }
}

fn choose_show_prepass_mode(
    mut commands: Commands,
    camera: Single<Entity, With<Camera3d>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        commands.entity(*camera).remove::<ShowPrepass>();
    } else if keyboard.just_pressed(KeyCode::F2) {
        commands.entity(*camera).insert(ShowPrepass::Depth);
    } else if keyboard.just_pressed(KeyCode::F3) {
        commands.entity(*camera).insert(ShowPrepass::Normals);
    } else if keyboard.just_pressed(KeyCode::F4) {
        commands.entity(*camera).insert(ShowPrepass::MotionVectors);
    }
}
