#[path = "camera_controller.rs"]
mod camera_controller;

use bevy::{
    core_pipeline::{Skybox, bloom::Bloom, tonemapping::Tonemapping},
    diagnostic::*,
    image::ImageLoaderSettings,
    pbr::light_consts::lux::DIRECT_SUNLIGHT,
    prelude::*,
    render::view::ColorGrading,
};
use bevy_feronia::prelude::Wind;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};
use camera_controller::*;
use iyes_perf_ui::prelude::*;

#[derive(Component)]
pub struct Landscape;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin,
                SystemInformationDiagnosticsPlugin,
                PerfUiPlugin,
            ))
            .add_plugins((
                EguiPlugin {
                    enable_multipass_for_primary_context: true,
                },
                ResourceInspectorPlugin::<Wind>::default(),
            ))
            .add_plugins(CameraControllerPlugin)
            .add_systems(Startup, setup);
    }
}

pub fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let diff_texture: Handle<Image> = asset_server.load("textures/brown_mud_leaves_01_diff_4k.jpg");
    let ao_texture: Handle<Image> = asset_server.load("textures/brown_mud_leaves_01_ao_4k.jpg");
    let normal_texture: Handle<Image> = asset_server.load_with_settings(
        "textures/brown_mud_leaves_01_nor_dx_4k.png",
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );
    let arm_texture: Handle<Image> = asset_server.load("textures/brown_mud_leaves_01_arm_4k.jpg");

    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        Landscape,
        MeshMaterial3d(materials.add(StandardMaterial {
            perceptual_roughness: 0.8,
            metallic: 0.0,
            base_color_texture: Some(diff_texture.clone()),
            metallic_roughness_texture: Some(arm_texture.clone()),
            occlusion_texture: Some(ao_texture.clone()),
            normal_map_texture: Some(normal_texture.clone()),
            ..default()
        })),
        Transform::default(),
    ));

    cmd.spawn((
        Camera {
            hdr: true,
            ..default()
        },
        Controller::default(),
        Camera3d::default(),
        ColorGrading::default(),
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(-10., 2., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: asset_server.load("skybox.ktx2"),
            brightness: 10000.,
            ..default()
        },
        /*
        Msaa::Off,
        bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
        bevy::core_pipeline::experimental::taa::TemporalAntiAliasing::default(),
        */
    ));

    cmd.spawn((
        Landscape,
        Mesh3d(meshes.add(Sphere::new(3.0).mesh().uv(120, 64))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 5.0, 0.5),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0., 5.0, 0.0),
    ))
    .with_child(PointLight {
        radius: 3.0,
        color: Color::srgb(0.1, 0.1, 1.0),
        ..default()
    });

    cmd.spawn((
        DirectionalLight {
            illuminance: DIRECT_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-50., 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    cmd.spawn(PerfUiDefaultEntries::default());
}
