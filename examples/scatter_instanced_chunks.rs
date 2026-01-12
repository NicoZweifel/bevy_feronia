#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_color::palettes::tailwind::*;
use bevy_eidolon::prelude::*;
use bevy_feronia::{
    asset::backend::scene_backend::SceneAssetBackendPlugin, instancing::scatter::scatter_layer,
    prelude::*,
};
use example::*;

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource, Clone)]
struct InstancedMaterialExampleConfig {
    wind: Wind,
    options: ScatterMaterialOptions,
}

fn main() -> AppExit {
    App::new()
        .register_type::<InstancedMaterialExampleConfig>()
        .insert_resource(GlobalWind {
            current: Wind {
                ..WindPreset::Mild.into()
            },
            ..WindPreset::Mild.into()
        })
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            show_debug_options: true,
            show_inspector: true,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            SceneAssetBackendPlugin,
            InstancedWindAffectedScatterPlugin,
            GpuComputeCullPlugin,
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
                InstanceScale,
                InstanceJitter,
                ScatterChunked,
                ScaleDensity,
                InstanceRotationYaw,
            ),
            // Material options
            (
                WindAffected,
                InstanceColor::new(YELLOW_950),
                InstanceColorGradient {
                    end: 0.6,
                    start: 0.0,
                    ..InstanceColorGradient::new(YELLOW_200, PURPLE_950)
                },
                EdgeCorrection,
                AnalyticalNormals,
                StaticBend,
                CurveNormals,
                DirectionalLights,
                PointLights,
                GpuCullCompute,
                AmbientOcclusion,
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
