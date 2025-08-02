#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::prelude::*;
use bevy_feronia::chunking::plugin::ChunkPlugin;
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
            high_quality: true,
            lod_threshold: 10.0,
            ..default()
        })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![RED_500.into(), ORANGE_500.into(), YELLOW_500.into()],
            aabb_color: GREEN_500.into(),
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((
            ExamplePlugin,
            WindPlugin,
            InstancedWindAffectedPlugin,
            ChunkPlugin,
            ScatterPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
        WindAffected,
    ));
    cmd.spawn((SceneRoot(assets.load("grass.glb#Scene0")), WindAffected));
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat_large.glb#Scene0")),
        ScatterRoot::default(),
        ChunkConfig::default(),
        children![(
            scatter_layer("Wind affected Foliage Layer"),
            DistributionDensity(100.0),
        )],
    ))
    .observe(scatter_observer);
}

fn scatter_on_keypress(
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Query<Entity, With<ScatterRoot>>,
    mut ew_scatter: EventWriter<Scatter>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    println!("Scattering plants...");

    for e in &q_root {
        ew_scatter.write(Scatter(e));
    }
}
