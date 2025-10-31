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
            // Scatter Options
            DistributionDensity(50.),
            InstanceJitter::default(),
            // You can define material options on the full layer here.
            WindAffected,
            children![
                (
                    // Or overwrite on the item, e.g.,
                    // WindAffected,
                    // CAUTION: If you have multiple assets, all lods that belong to each other need to have the same name!

                    // You can have multiple types in each layer; as long as all LODs have the same name, they will be matched correctly.
                    Name::new("Wind Affected Item"),
                    MeshMaterial3d(materials.add(StandardMaterial::default())),
                    Mesh3d(mesh.clone()),
                ),
                (
                    Name::new("Wind Affected Item"),
                    // We need to specify the LOD Level if it is not 0 (Highest level)
                    LevelOfDetail(1),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: RED_500.into(),
                        ..default()
                    })),
                    Mesh3d(mesh.clone()),
                ),
                (
                    Name::new("Wind Affected Item"),
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
