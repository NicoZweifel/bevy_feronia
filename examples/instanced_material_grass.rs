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
            strength: 0.8,
            micro_strength: 0.2,
            round_exponent: 15.,
            edge_correction_factor: 0.001,
            ..default()
        })
        .insert_resource(ExamplePluginOptions {
            no_indirect_drawing: true,
        })
        .add_plugins((ExamplePlugin, WindPlugin, InstancedWindAffectedPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, scatter_on_keypress)
        .run()
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.spawn((SceneRoot(assets.load("grass.glb#Scene0")), WindAffected));
    cmd.spawn((
        SceneRoot(assets.load("grass_low_lod.glb#Scene0")),
        WindAffected,
    ));
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

    let grid_size = 500;
    let cell_size = 0.08;
    let plant_offset = 0.04;
    let grid_world_size = grid_size as f32 * cell_size;

    let mut rng = rand::rng();

    q.iter().for_each(|x| cmd.entity(x).despawn());

    for prototype in prototypes.get().iter() {
        cmd.spawn((
            InstancedWindAffectedMaterial::component(prototype.material.clone()),
            WindAffected,
            WindAffectedReady,
            Mesh3d(prototype.mesh.clone()),
            InstanceMaterialData(
                (0..grid_size * grid_size)
                    .map(|i| {
                        let grid_x = (i % grid_size) as f32;
                        let grid_z = (i / grid_size) as f32;

                        let x = grid_x * cell_size - grid_world_size / 2.0;
                        let z = grid_z * cell_size - grid_world_size / 2.0;

                        let x_jitter = rng.random_range(-plant_offset..plant_offset);
                        let z_jitter = rng.random_range(-plant_offset..plant_offset);

                        InstanceData {
                            position: Vec3::new(x + x_jitter, 0.0, z + z_jitter),
                            scale: 1.,
                            color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0))
                                .to_f32_array(),
                            index: i,
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            NoFrustumCulling,
        ));
    }
}
