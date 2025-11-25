#[path = "camera_controller.rs"]
mod camera_controller;

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
    light::{CascadeShadowConfigBuilder, VolumetricLight},
    prelude::*,
    render::view::ColorGrading,
};
use bevy_feronia::prelude::*;
use bevy_feronia::quality::QualitySettings;
use bevy_image::{ImageSampler, ImageSamplerDescriptor};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};
use bevy_render::diagnostic::RenderDiagnosticsPlugin;
use bevy_render::view::Hdr;
use camera_controller::*;
use iyes_perf_ui::prelude::*;

#[derive(Resource, Default, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ExamplePluginOptions {
    pub show_quality_settings: bool,
    pub show_wind_settings: bool,
    pub show_inspector: bool,
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
            .add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
                PerfUiPlugin,
            ))
            .add_plugins((
                EguiPlugin::default(),
                WorldInspectorPlugin::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_inspector),
                ResourceInspectorPlugin::<Wind>::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_wind_settings),
                ResourceInspectorPlugin::<QualitySettings>::default()
                    .run_if(|res: Res<ExamplePluginOptions>| res.show_quality_settings),
            ))
            .add_plugins(CameraControllerPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (anisotropic_filtering, rotate_sun));
    }
}

pub fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    // options: Res<ExamplePluginOptions>,
) {
    cmd.spawn((
        Camera::default(),
        Hdr,
        Controller::default(),
        Camera3d::default(),
        ColorGrading::default(),
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(-30., 20., 30.).looking_at(Vec3::ZERO, Vec3::Y),
        ChunkCenter,
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
        CascadeShadowConfigBuilder::default().build(),
        Transform::from_xyz(2., 2., 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    cmd.spawn(PerfUiDefaultEntries::default());
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
