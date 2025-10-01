#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::extension::observers::extended_scatter_observer;
use bevy_feronia::extension::scatter::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            round_exponent: 15.,
            edge_correction_factor: 0.01,
            ..default()
        })
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
            InstanceJitter(1.),
            WindAffected,
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(1),
                )
            ]
        )],
    ))
    .observe(extended_scatter_observer);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Single<Entity, With<ScatterRoot>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    cmd.trigger(Scatter::<StandardMaterial, ExtendedWindAffectedMaterial>::new(*q_root))
}


