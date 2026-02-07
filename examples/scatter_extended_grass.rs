#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy_feronia::asset::backend::mesh_material_backend::MeshMaterialAssetBackendPlugin;
use bevy_feronia::extension::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(GlobalWind::from(WindPreset::Mild))
        .insert_resource(ExamplePluginOptions {
            show_wind_settings: true,
            ..default()
        })
        .add_plugins((
            ExamplePlugin,
            // This example spawns everything in startup, so we can just use the MeshMaterialAssetBackendPlugin
            MeshMaterialAssetBackendPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .insert_state(ScatterState::Setup)
        .insert_state(HeightMapState::Setup)
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress.in_set(ScatterSet::Ready))
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((
        SceneRoot(assets.load("landscape_flat.glb#Scene0")),
        ScatterRoot::default(),
        children![(
            scatter_layer("Foliage Layer"),
            // Scatter options
            (DistributionDensity(100.), InstanceJitterStrength::default(),),
            // Material options
            (
                WindAffected,
                SubsurfaceScattering,
                CurveNormals,
                // This example is mainly for testing lighting and bill boarding.
                // AnalyticalNormals, // or
                // FastNormals,
                InstanceRotationYaw,
                // or
                // EnableBillboarding,
            ),
            children![
                SceneRoot(assets.load("grass.glb#Scene0")),
                (
                    SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
                    LevelOfDetail(1),
                )
            ]
        )],
    ));
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    root: Single<Entity, With<ScatterRoot>>,
    mut world_seed: ResMut<WorldSeed>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    **world_seed = rng().next_u64();

    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root))
}
