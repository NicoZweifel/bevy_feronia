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
            strength: 0.05,
            micro_strength: 0.03,
            s_curve_strength: 0.01,
            bop_strength: 0.01,
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
        LodConfig(vec![LodLevelDistance::default()]),
        children![(
            scatter_layer("Foliage Layer"),
            DistributionDensity(15.),
            InstanceRotationYaw {
                min: 0.,
                max: std::f32::consts::PI * 2.
            },
            InstanceScale { min: 1., max: 3. },
            InstanceJitter(1.),
            WindAffected,
            children![SceneRoot(assets.load("foliage.glb#Scene0"))]
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

    cmd.trigger(
        q_root.iter().map(|x|
        Scatter::<StandardMaterial, ExtendedWindAffectedMaterial>::new(x)).collect(),
    );
}
