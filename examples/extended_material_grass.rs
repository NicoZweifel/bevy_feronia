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
            strength: 0.8,
            micro_strength: 0.2,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            ..default()
        })
        .add_plugins((
            ScatterAssetPlugin::<StandardMaterial, ExtendedWindAffectedMaterial>::new(),
            ExamplePlugin,
            WindPlugin,
            ExtendedWindAffectedPlugin,
            ScatterPlugin::<StandardMaterial, ExtendedWindAffectedMaterial>::new(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Wind affected Foliage Layer"),
            DistributionDensity(70.0),
            InstanceJitter(0.1),
            WindAffected,
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LodLevel(1),
                )
            ]
        )],
    ))
    .observe(extended_scatter_observer);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Query<Entity, With<ScatterRoot>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    cmd.trigger_targets(
        Scatter::<StandardMaterial, ExtendedWindAffectedMaterial>::new(),
        q_root.iter().collect::<Vec<_>>(),
    );
}
