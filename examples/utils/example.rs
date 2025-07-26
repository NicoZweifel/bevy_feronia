#[path = "camera_controller.rs"]
mod camera_controller;

use bevy::render::view::Hdr;
use bevy::{
    core_pipeline::{Skybox, bloom::Bloom, tonemapping::Tonemapping}, //diagnostic::*,
    pbr::light_consts::lux::DIRECT_SUNLIGHT,
    prelude::*,
    render::view::{ColorGrading, NoIndirectDrawing},
};
use bevy_feronia::prelude::*;
use camera_controller::*;

#[derive(Component)]
pub struct Landscape;

#[derive(Resource, Default)]
pub struct ExamplePluginOptions {
    // TODO remove this when using draw_indirect_indexed.
    // needs to be true for draw_indexed to work if not using chunking
    pub no_indirect_drawing: bool,
}

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExamplePluginOptions>()
            .add_plugins(DefaultPlugins.set(AssetPlugin { ..default() }))
            .add_plugins(CameraControllerPlugin)
            .add_systems(Startup, setup);
    }
}

pub fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    options: Res<ExamplePluginOptions>,
) {
    cmd.spawn((SceneRoot(asset_server.load("landscape.glb#Scene0")),));

    let camera = cmd
        .spawn((
            Camera::default(),
            Hdr,
            Controller::default(),
            Camera3d::default(),
            ColorGrading::default(),
            Bloom::NATURAL,
            Tonemapping::TonyMcMapface,
            Transform::from_xyz(-10., 2., 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ChunkCenter,
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
        ))
        .id();

    if options.no_indirect_drawing {
        cmd.entity(camera).insert(NoIndirectDrawing);
    }

    cmd.spawn((
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
}
