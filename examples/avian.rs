/// This example showcases how to integrate avian.
///
/// Arguably this is a bit experimental and will be changed heavily soon.
/// In a real scenario you would make use of [`CollisionLayers`] to prevent scattered entities from affecting each other,
/// which stops them tanking the performance like crazy.
///
/// In a level where we need to scatter around existing physics objects, we would have to populate the [`AvoidanceData`] component.
///
/// TODO
/// https://github.com/NicoZweifel/bevy_feronia/issues/56
/// https://github.com/NicoZweifel/bevy_feronia/issues/43
#[path = "utils/example.rs"]
mod example;

use avian3d::PhysicsPlugins;
use avian3d::prelude::{ColliderConstructor, PhysicsDebugPlugin, RigidBody};
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
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
            ExtendedWindAffectedScatterPlugin,
        ))
        .insert_state(HeightMapState::Setup)
        .insert_state(ScatterState::Setup)
        .add_systems(Startup, setup)
        .add_observer(
            |trigger: On<Add, ScatteredAsset<ExtendedWindAffectedMaterial>>,
             q_instance: Query<(Entity, &ScatteredAsset<ExtendedWindAffectedMaterial>)>,
             assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
             mut cmd: Commands| {
                let (entity, asset) = q_instance.get(trigger.entity).unwrap();
                let asset = assets.get(&**asset).unwrap();

                if let Some(rigid_body) = asset.rigid_body {
                    cmd.entity(entity).insert(rigid_body);
                }
            },
        )
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
        RigidBody::Static,
        ColliderConstructor::ConvexHullFromMesh,
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
        ScatterRoot::default(),
        children![(
            // Make sure you use the correct `ScatterLayer` with the desired `ScatterLayerType`, e.g.,
            // Standard, Extended or Instanced Material/Layer.
            extension::scatter_layer("Wind Affected Layer"),
            // Scatter Options
            DistributionDensity(20.),
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
                    RigidBody::Static,
                    ColliderConstructor::ConvexHullFromMesh,
                    Transform::from_xyz(0., 2.5, 0.),
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
                    RigidBody::Static,
                    ColliderConstructor::ConvexHullFromMesh,
                    Transform::from_xyz(0., 2.5, 0.),
                ),
                (
                    Name::new("Wind Affected Asset"),
                    LevelOfDetail(2),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: GREEN_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh),
                    RigidBody::Static,
                    ColliderConstructor::ConvexHullFromMesh,
                    Transform::from_xyz(0., 2.5, 0.),
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
