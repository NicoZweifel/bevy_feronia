#[path = "utils/example.rs"]
mod example;

use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy_feronia::prelude::*;
use example::*;
use rand::Rng;
use rand::seq::IndexedRandom;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            strength: 1.0,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            ..default()
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((ExamplePlugin, WindPlugin, InstancedWindAffectedPlugin))
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
    prototypes: Res<WindAffectedTypes<InstancedWindAffectedMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q: Query<Entity, With<WindAffected>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    println!("Scattering plants...");

    q.iter().for_each(|x| cmd.entity(x).despawn());

    const CHUNK_GRID_SIZE: u32 = 10;
    const INSTANCES_PER_CHUNK_DIM: u32 = 50;
    const CELL_SIZE: f32 = 0.04;
    const PLANT_OFFSET: f32 = 0.02;

    let mut rng = rand::rng();

    let instances_per_chunk = INSTANCES_PER_CHUNK_DIM.pow(2);
    let chunk_world_size = INSTANCES_PER_CHUNK_DIM as f32 * CELL_SIZE;
    let total_world_size = CHUNK_GRID_SIZE as f32 * chunk_world_size;
    let world_half_size = total_world_size / 2.0;

    let prototype = prototypes.get().choose(&mut rng).unwrap();

    for chunk_z in 0..CHUNK_GRID_SIZE {
        for chunk_x in 0..CHUNK_GRID_SIZE {
            let chunk_origin = Vec3::new(
                chunk_x as f32 * chunk_world_size - world_half_size,
                0.0,
                chunk_z as f32 * chunk_world_size - world_half_size,
            );

            let instance_data = (0..instances_per_chunk)
                .map(|i| {
                    let local_instance_x = (i % INSTANCES_PER_CHUNK_DIM) as f32;
                    let local_instance_z = (i / INSTANCES_PER_CHUNK_DIM) as f32;

                    let pos_in_chunk = Vec3::new(
                        local_instance_x * CELL_SIZE,
                        0.0,
                        local_instance_z * CELL_SIZE,
                    );

                    let jitter = Vec3::new(
                        rng.random_range(-PLANT_OFFSET..PLANT_OFFSET),
                        0.0,
                        rng.random_range(-PLANT_OFFSET..PLANT_OFFSET),
                    );

                    InstanceData {
                        position: chunk_origin + pos_in_chunk + jitter,
                        scale: 1.0,
                        color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                        index: i,
                    }
                })
                .collect::<Vec<_>>();

            cmd.spawn((
                WindAffected,
                InstancedWindAffectedMeshMaterial(prototype.material.clone()),
                InstanceMaterialData(instance_data),
                WindAffectedReady,
                Mesh3d(prototype.mesh.clone()),
                NoFrustumCulling,
            ));
        }
    }
}
