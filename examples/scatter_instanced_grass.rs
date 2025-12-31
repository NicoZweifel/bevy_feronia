#[path = "utils/example.rs"]
mod example;

use example::*;

use bevy::prelude::*;
use bevy_color::palettes::tailwind::{BLUE_400, RED_400};
use bevy_feronia::{
    asset::backend::mesh_material_backend::MeshMaterialAssetBackendPlugin,
    instancing::scatter_layer, prelude::*,
};
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            // This example spawns everything in startup, so we can just use the MeshMaterialAssetBackendPlugin
            MeshMaterialAssetBackendPlugin,
            InstancedWindAffectedScatterPlugin,
        ))
        .add_systems(Startup, setup)
        .insert_state(HeightMapState::Setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        Transform::from_xyz(5., 0., 0.)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
        ScatterRoot::default(),
        children![(
            scatter_layer("Grass Layer"),
            // Scatter Options
            (
                DistributionDensity(100.),
                InstanceScale { min: 1., max: 1.8 },
                InstanceJitter::default()
            ),
            // Material Options
            (
                WindAffected,
                EdgeCorrectionFactor::default(),
                CurveFactor::default(),
                StaticBendStrength::default(),
                AnalyticalNormals,
                PointLights,
                DirectionalLights,
                InstanceColorGradient::new(RED_400.into(), BLUE_400.into())
            ),
            children![
                (SceneRoot(assets.load("grass.glb#Scene0")), LevelOfDetail(0),),
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
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    **world_seed = rng().next_u64();

    cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}
