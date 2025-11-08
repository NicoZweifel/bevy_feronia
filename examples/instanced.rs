#[path = "utils/example.rs"]
mod example;

use bevy::camera::primitives::Aabb;
use bevy::color::palettes::tailwind::*;
use bevy::mesh::{Indices, PlaneMeshBuilder, PrimitiveTopology};
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
    let mesh_handle = meshes.add(create_triangle_with_foliage_uvs());
    let aabb = Aabb {
        center: Vec3A::new(0.25, 0.375, 0.0),
        half_extents: Vec3A::new(0.25, 1.125, 0.0),
    };

    cmd.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY_500.into(),
            ..default()
        })),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
    ));

    let options = MaterialOptions {
        // make it affected by wind
        wind_affected: true,
        // can also tweak other settings here
        analytical_normals: true,
        curve_factor: 0.2,
        ..default()
    };

    let material_handle = instanced_materials.add(InstancedWindAffectedMaterial {
        wind: *wind,
        aabb,
        options,
        noise_texture: (**noise_texture).clone(),
    });

    let instances = (0..10)
        .map(|x| InstanceData {
            position: Vec3::new(x as f32, 0.75 * 4., x as f32),
            scale: 4.0,
            index: 0,
            ..default()
        })
        .collect();

    let instance_material_data = InstanceMaterialData {
        color: GREEN_500.to_f32_array(),
        visibility_range: [0.0, 0.0, 1000.0, 1000.0],
        instances,
        static_bend_strength: options.static_bend_strength,
        curve_factor: options.curve_factor,
    };

    cmd.spawn((
        InstancedWindAffectedMeshMaterial(material_handle),
        Mesh3d(mesh_handle),
        instance_material_data,
        NoAutomaticBatching,
        Transform::default(),
        Visibility::Visible,
        // Disable frustum culling or provide aabb.
        // NoFrustumCulling,
        Aabb {
            center: aabb.center,
            half_extents: aabb.half_extents * 10.,
        },
    ));
}

/// Creates a triangle mesh UVs.
fn create_triangle_with_foliage_uvs() -> Mesh {
    let positions = vec![[0.0, -0.75, 0.0], [0.0, 1.5, 0.0], [0.5, -0.75, 0.0]];

    let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];

    let uvs = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];

    let indices = Indices::U32(vec![0, 1, 2]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    // Generate tangents, which are required by the shader for normal mapping and the curve effect.
    mesh.generate_tangents().unwrap();
    mesh.insert_indices(indices);
    mesh
}
