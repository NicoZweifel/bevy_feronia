#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::css::WHITE;
use bevy::color::palettes::tailwind::{GREEN_500, ORANGE_500, RED_500, YELLOW_500};
use bevy::prelude::*;
use bevy_feronia::instancing::observers::instanced_scatter_observer;
use bevy_feronia::instancing::scatter::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .insert_resource(ChunkDebugConfig {
            lod_colors: vec![
                RED_500.into(),
                ORANGE_500.into(),
                YELLOW_500.into(),
                WHITE.into(),
            ],
            aabb_color: GREEN_500.into(),
        })
        .add_plugins((ExamplePlugin, InstancedWindAffectedScatterPlugin))
        .insert_state(ScatterState::Setup)
        .insert_state(HeightMapState::Setup)
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat_large.glb#Scene0")),
        ChunkRoot::default(),
        ScatterRoot::default(),
        MapHeight,
        children![(
            scatter_layer("Grass Layer"),
            DistributionDensity(80.0),
            InstanceScale { min: 1., max: 2. },
            InstanceJitter(1.),
            WindAffected,
            ScaleDensity,
            ScatterChunked,
            EnableBillboarding,
            EdgeCorrectionFactor::default(),
            children![
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(2)
                ),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(1),
                ),
                (SceneRoot(assets.load("grass.glb#Scene0")),)
            ]
        )],
    ))
    .observe(instanced_scatter_observer);
}
