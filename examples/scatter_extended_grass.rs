#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::extension::scatter::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .add_plugins((ExamplePlugin, ExtendedWindAffectedScatterPlugin))
        .insert_state(ScatterState::Setup)
        .insert_state(HeightMapState::Setup)
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Foliage Layer"),
            DistributionDensity(100.),
            InstanceJitter::default(),
            WindAffected,
            EnableBillboarding,
            SubsurfaceScattering,
            CurveFactor::default(),
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(1),
                )
            ]
        )],
    ));
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Single<Entity, With<ScatterRoot>>,
    mut world_seed: ResMut<WorldSeed>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    **world_seed = rng().next_u64();

    println!("Scattering");

    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*q_root))
}
