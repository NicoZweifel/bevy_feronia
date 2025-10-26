#[path = "camera_controller.rs"]
mod camera_controller;

use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::post_process::bloom::Bloom;
use bevy::render::view::Hdr;
use bevy::{
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    light::{CascadeShadowConfigBuilder, VolumetricLight},
    prelude::*,
    render::view::ColorGrading,
};
use bevy_feronia::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};
use camera_controller::*;

#[cfg(not(feature = "dlss"))]
use bevy::anti_alias::taa::TemporalAntiAliasing;
#[cfg(all(feature = "dlss"))]
use bevy::{
    anti_alias::dlss::{Dlss, DlssPerfQualityMode, DlssProjectId},
    asset::uuid,
};
use bevy::diagnostic::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use iyes_perf_ui::prelude::*;

#[derive(Resource, Default)]
pub struct ExamplePluginOptions {
    // add options here if needed again
}

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(feature = "dlss"))]
        app.insert_resource(DlssProjectId(uuid::uuid!(
            "edac5c37-87f0-4e5c-be93-3636dd13677a"
        )));

        app.init_resource::<ExamplePluginOptions>()
            .add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                RenderDiagnosticsPlugin,
                SystemInformationDiagnosticsPlugin,
                PerfUiPlugin,
            ))
            .add_plugins((
                EguiPlugin::default(),
                ResourceInspectorPlugin::<Wind>::default(),
            ))
            .add_plugins(CameraControllerPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, anisotropic_filtering);
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
        #[cfg(all(feature = "dlss"))]
        (
            Msaa::Off,
            Dlss {
                perf_quality_mode: DlssPerfQualityMode::Dlaa,
                ..default()
            },
        ),
        #[cfg(not(feature = "dlss"))]
        (
            Msaa::Off,
            bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
            TemporalAntiAliasing::default(),
        ),
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
        radius: 3.,
        color: Color::srgb(0.1, 0.1, 1.),
        ..default()
    });

    cmd.spawn((
        DirectionalLight {
            // NOTE: Direct sunlight has over-exposure with the SkyBox, FULL_DAYLIGHT seems a bit low but 30_000. seems fine.
            illuminance: 30_000.,
            shadows_enabled: true,
            ..default()
        },
        VolumetricLight,
        CascadeShadowConfigBuilder {
            maximum_distance: 300.,
            ..default()
        }
        .build(),
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
