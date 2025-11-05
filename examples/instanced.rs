#[path = "utils/example.rs"]
mod example;

use bevy::camera::primitives::Aabb;
use bevy::color::palettes::tailwind::*;
use bevy::mesh::PlaneMeshBuilder;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use bevy_feronia::prelude::*;
use bevy_feronia::wind::systems::setup_wind_texture;
use example::*;

fn main() -> AppExit {
    App::new()
        .init_resource::<Wind>()
        .add_plugins((
            ExamplePlugin,
            // Don't need any of the scatter plugins if we just want to have wind-affected materials,
            // but we do need to add the `WindPlugin` manually.
            //
            // Some features that depend on the `ScatterAssets` won't work automatically e.g.,
            // syncing the wind of the materials with the resource.
            WindPlugin,
            InstancedWindAffectedPlugin,
        ))
        // Need to wait for the wind noise texture.
        .add_systems(Startup, setup.after(setup_wind_texture))
        .run()
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut instanced_materials: ResMut<Assets<InstancedWindAffectedMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    wind: Res<Wind>,
    noise_texture: Res<WindTexture>,
) {
    let mesh_handle = meshes.add(Cuboid::new(0.5, 3.0, 0.5));
    let aabb = Aabb {
        half_extents: Vec3A::new(0.25, 1.5, 0.25),
        center: Vec3A::new(0.0, 0.0, 0.0),
    };

    cmd.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY_500.into(),
            ..default()
        })),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
    ));

    let material_handle = instanced_materials.add(InstancedWindAffectedMaterial {
        wind: *wind,
        aabb,
        options: MaterialOptions {
            // make it affected by wind
            wind_affected: true,
            // can also tweak other settings here
            ..default()
        },
        noise_texture: (**noise_texture).clone(),
    });

    let instances = (0..10)
        .map(|x| InstanceData {
            position: Vec3::new(x as f32, 1.5, 0.0),
            scale: 1.0,
            index: 0,
            ..default()
        })
        .collect();

    let instance_material_data = InstanceMaterialData {
        color: GREEN_500.to_f32_array(),
        visibility_range: [0.0, 0.0, 1000.0, 1000.0],
        instances,
        static_bend_strength: 0.1,
        curve_factor: 0.2,
    };

    cmd.spawn((
        InstancedWindAffectedMeshMaterial(material_handle),
        Mesh3d(mesh_handle),
        instance_material_data,
        NoAutomaticBatching,
        Transform::default(),
        Visibility::Visible,
        aabb,
    ));
}
