#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::instancing::observers::scatter_observer;
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
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            InstancedWindAffectedPlugin,
            ScatterPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((SceneRoot(assets.load("grass.glb#Scene0")), WindAffected));
    cmd.spawn((
        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
        WindAffected,
    ));

    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Wind affected Foliage Layer"),
            DistributionDensity(150.0),
            InstanceRotationYaw {
                min: 0.0,
                max: std::f32::consts::PI * 2.0
            },
            InstanceScale { min: 1., max: 3. },
            InstanceJitter(0.1)
        )],
    ))
    .observe(scatter_observer);
}

fn scatter_on_keypress(
    mut cmd: Commands,
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Query<Entity, With<ScatterRoot>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    println!("Scattering plants...");

    cmd.trigger_targets(
        Scatter::<ScatterRoot>::default(),
        q_root.iter().collect::<Vec<_>>(),
    );
}
