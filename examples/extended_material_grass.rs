#[path = "utils/example.rs"]
mod example;

use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_feronia::prelude::*;
use example::*;
use rand::Rng;
use rand::seq::IndexedRandom;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            round_exponent: 80.,
            ..default()
        })
        .add_plugins((ExamplePlugin, ExtendedMaterialWindPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (init_grass, scatter_on_keypress))
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn(SceneRoot(assets.load("grass.glb#Scene0")));
}

fn init_grass(
    mut cmd: Commands,
    q: Query<
        Entity,
        (
            With<MeshMaterial3d<StandardMaterial>>,
            With<Mesh3d>,
            Without<Landscape>,
            Without<WindAffected>,
        ),
    >,
) {
    for e in &q {
        cmd.entity(e).insert(WindAffected);
    }
}

fn scatter_on_keypress(
    mut cmd: Commands,
    prototypes: Res<WindAffectedTypes<WindAffectedExtendedMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q: Query<Entity, With<MeshMaterial3d<WindAffectedExtendedMaterial>>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    println!("Scattering plants...");

    let grid_size = 260;
    let cell_size = 0.075;
    let plant_offset = 0.0375;

    let mut rng = rand::rng();

    q.iter().for_each(|x| cmd.entity(x).despawn());

    let grid_world_size = grid_size as f32 * cell_size;

    let batch = (0..grid_size * grid_size)
        .map(|i| {
            let prototype = prototypes.get().choose(&mut rng).unwrap();

            let grid_x = (i % grid_size) as f32;
            let grid_z = (i / grid_size) as f32;

            let x = grid_x * cell_size - grid_world_size / 2.0;
            let z = grid_z * cell_size - grid_world_size / 2.0;

            let x_jitter = rng.random_range(-plant_offset..plant_offset);
            let z_jitter = rng.random_range(-plant_offset..plant_offset);

            let y_rotation = rng.random_range(0.0..std::f32::consts::PI * 2.0);

            (
                Mesh3d(prototype.mesh.clone()),
                MeshMaterial3d(prototype.material.clone()),
                Transform::from_xyz(x + x_jitter, 0.0, z + z_jitter)
                    .with_rotation(Quat::from_rotation_y(y_rotation))
                    .with_scale(Vec3::splat(1.).with_y(rng.random_range((1.)..2.))),
                WindAffectedReady,
                NotShadowCaster,
            )
        })
        .collect::<Vec<_>>();

    cmd.spawn_batch(batch);
}
