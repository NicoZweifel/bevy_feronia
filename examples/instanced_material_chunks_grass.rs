#[path = "utils/example.rs"]
mod example;

use bevy::color::palettes::tailwind::GREEN_500;
use bevy::prelude::*;
use bevy::prelude::Visibility::Visible;
use bevy::render::batching::NoAutomaticBatching;
use bevy::render::primitives::{Aabb, MeshAabb};
use bevy::render::render_resource::ShaderType;
use bevy_feronia::prelude::*;
use example::*;
use rand::Rng;
use rand::seq::IndexedRandom;

fn main() -> AppExit {
    App::new()
        .insert_resource(Wind {
            enable_billboarding: true,
            enable_edge_correction: true,
            strength: 0.8,
            micro_strength: 0.2,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            enable_lod: true,
            lod_threshold: 10.0,
            ..default()
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((ExamplePlugin, WindPlugin, InstancedWindAffectedPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (init_grass, scatter_on_keypress, draw_aabbs))
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
    meshes: Res<Assets<Mesh>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    };

    if prototypes.get().is_empty() {
        println!("No plants found to scatter!");
        return;
    }

    q.iter().for_each(|e| cmd.entity(e).despawn());

    println!("Scattering plants...");

    const CHUNK_GRID_DIM: u32 = 5;
    const INSTANCES_PER_CHUNK_DIM: u32 = 100;
    const CELL_SIZE: f32 = 0.04;
    const JITTER_AMOUNT: f32 = 0.02;

    let mut rng = rand::rng();

    let Some(prototype) = prototypes.get().choose(&mut rng) else {
        return;
    };

    let total_instances_dim = CHUNK_GRID_DIM * INSTANCES_PER_CHUNK_DIM;
    let total_world_dim = total_instances_dim as f32 * CELL_SIZE;
    let center_offset = total_world_dim / 2.0;

    for chunk_z in 0..CHUNK_GRID_DIM {
        for chunk_x in 0..CHUNK_GRID_DIM {
            let instances = (0..INSTANCES_PER_CHUNK_DIM * INSTANCES_PER_CHUNK_DIM)
                .map(|i| {
                    let local_instance_x = i % INSTANCES_PER_CHUNK_DIM;
                    let local_instance_z = i / INSTANCES_PER_CHUNK_DIM;

                    let global_instance_x = (chunk_x * INSTANCES_PER_CHUNK_DIM) + local_instance_x;
                    let global_instance_z = (chunk_z * INSTANCES_PER_CHUNK_DIM) + local_instance_z;

                    let world_x = global_instance_x as f32 * CELL_SIZE - center_offset;
                    let world_z = global_instance_z as f32 * CELL_SIZE - center_offset;

                    let x_jitter = rng.random_range(-JITTER_AMOUNT..JITTER_AMOUNT);
                    let z_jitter = rng.random_range(-JITTER_AMOUNT..JITTER_AMOUNT);

                    InstanceData {
                        position: Vec3::new(world_x + x_jitter, 0.0, world_z + z_jitter),
                        scale: 1.0,
                        color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                        index: i,
                    }
                })
                .collect::<Vec<_>>();

            let mesh_aabb = meshes.get(&prototype.mesh).unwrap().compute_aabb().unwrap();

            let mut min_point = Vec3::MAX;
            let mut max_point = Vec3::MIN;
            for instance in &instances {
                let instance_min = instance.position + <Vec3A as Into<Vec3>>::into(mesh_aabb.min());
                let instance_max = instance.position + <Vec3A as Into<Vec3>>::into(mesh_aabb.max());

                min_point = min_point.min(instance_min);
                max_point = max_point.max(instance_max);
            }

            cmd.spawn((
                InstancedWindAffectedMaterial::component(prototype.material.clone()),
                Mesh3d(prototype.mesh.clone()),
                InstanceMaterialData(instances),
                NoAutomaticBatching,
                WindAffectedReady,
                Aabb::from_min_max(min_point, max_point),
                Transform::default(),
                Visible,
                GlobalTransform::default(),
                ViewVisibility::default()
            ));
        }
    }
}

fn draw_aabbs(
    mut gizmos: Gizmos,
    query: Query<&Aabb, With<InstanceMaterialData>>,
) {
    for aabb in &query {
        let transform = Transform::from_translation(aabb.center.into())
            .with_scale(<Vec3A as Into<Vec3>>::into(aabb.half_extents) * Vec3::splat(2.0));

        gizmos.cuboid(transform, GREEN_500);
    }
}
