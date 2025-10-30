#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::*;
use bevy::mesh::PlaneMeshBuilder;
use bevy::prelude::*;
use bevy_feronia::extension::scatter::scatter_layer;
use bevy_feronia::prelude::*;
use example::*;
use rand::{RngCore, rng};

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind { ..default() })
        .add_plugins((ExamplePlugin, ExtendedWindAffectedScatterPlugin))
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
            scatter_layer("Wind Affected Layer"),
            DistributionDensity(50.),
            InstanceJitter::default(),
            children![
                (
                    // Only make lod 0 wind affected, this will make the scattered instances use a Material Extension for displacement.
                    WindAffected,
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    Mesh3d(mesh.clone()),
                ),
                (
                    LevelOfDetail(1),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: RED_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh.clone()),
                ),
                (
                    LevelOfDetail(2),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: GREEN_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh),
                )
            ]
        )],
    ));
}

fn scatter_on_keypress(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_root: Single<Entity, With<ScatterRoot>>,
    mut world_seed: ResMut<WorldSeed>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    **world_seed = rng().next_u64();

    println!("Scattering");

    cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*q_root))
}
