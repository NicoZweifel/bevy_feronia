#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::instancing::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .add_plugins((ExamplePlugin, InstancedWindAffectedScatterPlugin))
        .add_systems(Startup, setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Update, (scatter_on_keypress,))
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
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
