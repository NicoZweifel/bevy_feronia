#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::prelude::*;
use bevy_feronia::chunking::systems::debug::draw_aabbs;
use bevy_feronia::instancing::observers::instanced_scatter_observer;
use bevy_feronia::instancing::scatter::scatter_layer;
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
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into(), YELLOW_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .add_plugins((ExamplePlugin, InstancedWindAffectedScatterPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (scatter_on_keypress, draw_aabbs))
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Grass Layer"),
            DistributionDensity(100.0),
            InstanceScale { min: 1., max: 1.5 },
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
    .observe(instanced_scatter_observer);
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
        Scatter::<StandardMaterial, InstancedWindAffectedMaterial>::new(),
        q_root.iter().collect::<Vec<_>>(),
    );
}
