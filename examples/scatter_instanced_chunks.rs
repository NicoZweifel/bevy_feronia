#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::css::WHITE;
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
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
            // Scatter options
            (
                DistributionDensity(120.0),
                InstanceScale::default(),
                InstanceJitter(1.),
                ScaleDensity,
            ),
            // Material options
            (
                WindAffected,
                ScatterChunked,
                EdgeCorrectionFactor::default(),
                AnalyticalNormals,
                InstanceColor(Color::hsla(86., 0.69, 0.59, 1.0)),
                StaticBendStrength::default(),
                CurveFactor::default(),
                PointLights,
            ),
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_medium_lod.glb#Scene0")),
                    LevelOfDetail(1),
                ),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(2),
                )
            ]
        )],
    ));
}
