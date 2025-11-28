#[path = "utils/example.rs"]
mod example;

use bevy::app::prelude::*;
use bevy::prelude::*;
use bevy_color::palettes::tailwind::*;
use bevy_feronia::asset::backend::mesh_material_backend::MeshMaterialAssetBackendPlugin;
use bevy_feronia::extension;
use bevy_feronia::prelude::*;
use bevy_mesh::PlaneMeshBuilder;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .add_plugins((
            ExamplePlugin,
            MeshMaterialAssetBackendPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .insert_state(HeightMapState::Setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 5.0, 1.0));

    cmd.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY_500.into(),
            ..default()
        })),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
        ScatterRoot::default(),
        children![(
            // Make sure you use the correct `ScatterLayer` with the desired `ScatterLayerType`, e.g.,
            // Standard, Extended or Instanced Material/Layer.
            extension::scatter_layer("Wind Affected Layer"),
            // Scatter Options
            DistributionDensity(50.),
            InstanceJitter::default(),
            InstanceScale::default(),
            InstanceRotationYaw::default(),
            // You can define material options on the full layer here.
            WindAffected,
            children![
                (
                    // You can have multiple assets in each layer; as long as all LODs have the same name, they will be matched correctly.
                    Name::new("Wind Affected Asset"),
                    // Always need a mesh/material combo:
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: RED_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh.clone()),
                    Transform::from_xyz(0., 3., 0.),
                ),
                (
                    Name::new("Wind Affected Asset"),
                    // We need to specify the LOD Level if it is not 0 (Highest level)
                    LevelOfDetail(1),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: RED_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh.clone()),
                    Transform::from_xyz(0., 3., 0.),
                ),
                (
                    Name::new("Wind Affected Asset"),
                    LevelOfDetail(2),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: GREEN_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh),
                    Transform::from_xyz(0., 3., 0.),
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

    println!("Scattering");

    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root))
}
