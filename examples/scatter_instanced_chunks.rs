#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_camera::primitives::Aabb;
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
        .insert_resource(GlobalWind::from(WindPreset::Mild))
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
            GpuComputeCullCorePlugin,
            GpuCullComputePlugin::<InstancedWindAffectedMaterial>::default(),
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
        Aabb {
            half_extents: Vec3A::new(100., 0., 100.),
            center: Vec3A::new(0., 0., 0.),
        },
        ChunkRoot::default(),
        ScatterRoot::default(),
        Transform::from_xyz(20., 0., 0.)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
        children![(
            scatter_layer("Grass Layer"),
            // Scatter options
            (
                DistributionDensity(250.0),
                InstanceScale,
                InstanceJitter,
                ScatterChunked,
                ScaleDensity,
                InstanceRotationYaw,
            ),
            // Material options
            (
                WindAffected,
                InstanceColor::new(Srgba::hex("#1f3114").unwrap()),
                InstanceColorGradient {
                    end: 0.7,
                    start: 0.2,
                    ..InstanceColorGradient::new(
                        Srgba::hex("#3e6328").unwrap(),
                        Srgba::hex("#0f190a").unwrap()
                    )
                },
                StandardPbr,
                SubsurfaceScattering,
                // These are default values, but they are included here for clarity:
                Roughness(0.5),
                Metallic(0.0),
                Reflectance(0.1),
                // or when not using `StandardPbr`:`
                /*
                (
                    DirectionalLights,
                    PointLights,
                    SpecularPower(32.0),
                    SpecularStrength(0.2),
                    DiffuseScaling(1.0),
                ),
                */
                // Broken currently with temporal fx
                // EdgeCorrection,
                AnalyticalNormals,
                StaticBend,
                CurveNormals,
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
