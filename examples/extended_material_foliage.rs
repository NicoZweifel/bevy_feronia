#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::extension::observers::scatter_observer;
use bevy_feronia::prelude::*;
use example::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            strength: 0.5,
            micro_strength: 0.2,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            ExtendedWindAffectedPlugin,
            ScatterPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((SceneRoot(assets.load("foliage.glb#Scene0")), WindAffected));

    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Wind affected Foliage Layer"),
            DistributionDensity(20.0),
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
    prototypes: Res<WindAffectedTypes<ExtendedWindAffectedMaterial>>,
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
