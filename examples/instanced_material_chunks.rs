#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::prelude::*;
use bevy_feronia::instancing::observers::instanced_scatter_observer;
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
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat_large.glb#Scene0")),
        ChunkRoot::default(),
        ScatterRoot::default(),
        children![(
            scatter_layer("Grass Layer"),
            DistributionDensity(25.0),
            InstanceScale { min: 1., max: 1.5 },
            InstanceJitter(0.5),
            children![
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LodLevel(2),
                    ScatterSource,
                    WindAffected,
                ),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LodLevel(1),
                    ScatterSource,
                    WindAffected,
                ),
                (
                    SceneRoot(assets.load("grass.glb#Scene0")),
                    ScatterSource,
                    WindAffected,
                )
            ]
        )],
    ))
    .observe(instanced_scatter_observer);
}
