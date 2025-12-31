#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use bevy_feronia::{
    asset::backend::scene_backend::SceneAssetBackendPlugin, instancing::scatter::scatter_layer,
    prelude::*,
};
use example::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            show_debug_options: true,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            SceneAssetBackendPlugin,
            InstancedWindAffectedScatterPlugin,
        ))
        .insert_state(HeightMapState::Setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Startup, setup)
        .add_systems(Update, grass_count)
        .run()
}

fn grass_count(query: Query<&InstanceMaterialData>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyC) {
        println!(
            "{:?}",
            query
                .iter()
                .map(|x| x.instances.len() as u32)
                .fold(0, |acc, x| acc + x as usize)
        );
    }
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat_large.glb#Scene0")),
        ChunkRoot::default(),
        ScatterRoot::default(),
        Transform::from_xyz(20., 0., 0.)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
        children![(
            scatter_layer("Grass Layer"),
            // Scatter options
            (
                DistributionDensity(200.0),
                InstanceScale::default(),
                InstanceJitter(1.),
                ScatterChunked,
                ScaleDensity,
                InstanceRotationYaw::default(),
            ),
            // Material options
            (
                WindAffected,
                EdgeCorrectionFactor::default(),
                AnalyticalNormals,
                StaticBendStrength::default(),
                CurveFactor::default(),
                DirectionalLights,
                PointLights,
                GpuCullCompute
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
