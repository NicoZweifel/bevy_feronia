#[path = "utils/example.rs"]
mod example;

use example::*;

use bevy_feronia::instancing::prelude::*;
use bevy_feronia::prelude::*;

use bevy_app::{App, AppExit, Startup};
use bevy_asset::Assets;
use bevy_camera::primitives::Aabb;
use bevy_camera::visibility::{NoAutoAabb, Visibility};
use bevy_color::palettes::tailwind::*;
use bevy_ecs::prelude::*;
use bevy_eidolon::prelude::*;
use bevy_math::{Vec3, Vec3A};
use bevy_mesh::{Indices, Mesh, Mesh3d, MeshBuilder, PlaneMeshBuilder, PrimitiveTopology};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_render::batching::NoAutomaticBatching;
use bevy_transform::prelude::Transform;
use bevy_utils::default;

use std::sync::Arc;

fn main() -> AppExit {
    App::new()
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
        .add_systems(Startup, setup.after(WindTextureSetup))
        .run()
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut instanced_materials: ResMut<Assets<InstancedWindAffectedMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    global_wind: Res<GlobalWind>,
    noise_texture: Res<WindTexture>,
) {
    let wind = global_wind.current;
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

    let options = ScatterMaterialOptions {
        // make it affected by wind
        wind: WindOptions {
            affected: true,
            ..default()
        },
        // can also tweak or test other individual settings here
        geometry: GeometryOptions {
            edge_correction_factor: 1.0,
            ..default()
        },
        lighting: LightingOptions {
            common: CommonLightingOptions {
                directional_lights: true,
                point_lights: true,
                ambient_occlusion: true,
                ..default()
            },
            blinn_phong: BlinnPhongOptions {
                specular_power: 32.,
                specular_strength: 0.5,
                translucency: 0.6,
                light_intensity: 0.0001,
                ambient_light_intensity: 0.0001,
                diffuse_scaling: 1.,
            },
            ..default()
        },
        color: ColorOptions {
            top_color: Some(RED_400.into()),
            bottom_color: Some(BLUE_400.into()),
            base_color: Some(GREEN_400.into()),
            gradient_start: 0.3,
            gradient_end: 0.6,
            tint_factor: 0.5,
        },
        bend: StaticBendOptions {
            strength: 0.5,
            ..default()
        },
        ..default()
    };

    let color = options
        .color
        .base_color
        .unwrap_or_default()
        .clone()
        .to_linear();
    let material_handle = instanced_materials.add(InstancedWindAffectedMaterial {
        // Only clone the wind here in this example, if you want wind updates to be synced to the materials,
        // you need an update system.
        current: wind,
        previous: wind,
        aabb,
        options,
        noise_texture: (**noise_texture).clone(),
        disable_prepass: false,
    });

    const SIZE: i32 = 10;

    let instances = (-SIZE..SIZE)
        .enumerate()
        .map(|(i, x)| {
            InstanceData {
                position: Vec3::new(x as f32, 0.25 * 4., x as f32),
                scale: 4.0,
                // NOTE:
                // for testing, to remove rotation, pass the same index (e.g., 0) for all items here.
                index: i as u32,
                ..default()
            }
        })
        .collect();

    let instance_material_data = InstanceMaterialData {
        color,
        visibility_range: [0.0, 0.0, 1000.0, 1000.0].into(),
        instances: Arc::new(instances),
    };

    cmd.spawn((
        InstancedMeshMaterial(material_handle),
        Mesh3d(mesh_handle),
        instance_material_data,
        NoAutomaticBatching,
        Transform::default(),
        Visibility::Visible,
        // Disable frustum culling or provide aabb.
        // NoFrustumCulling,
        Aabb {
            center: aabb.center,
            half_extents: aabb.half_extents * SIZE as f32 * 2.,
        },
        // Required since bevy 0.18 if adding aabb manually.
        NoAutoAabb,
    ));
}

/// Creates a subdivided triangle mesh on the x and y-axis with normals/UVs.
fn create_triangle_with_foliage_uvs() -> Mesh {
    let pos_bottom_left = [0.0, -0.25, 0.0];
    let pos_top_center = [0.25, 0.5, 0.0];
    let pos_bottom_right = [0.5, -0.25, 0.0];

    let uv_bottom_left = [0.0, 1.0];
    let uv_top_center = [0.5, 0.0];
    let uv_bottom_right = [1.0, 1.0];

    // Midpoints
    let pos_bottom_center = [0.25, -0.25, 0.0];
    let pos_mid_left = [0.125, 0.125, 0.0];
    let pos_mid_right = [0.375, 0.125, 0.0];

    let uv_bottom_center = [0.5, 1.0];
    let uv_mid_left = [0.25, 0.5];
    let uv_mid_right = [0.75, 0.5];

    let positions = vec![
        pos_bottom_left,
        pos_top_center,
        pos_bottom_right,
        pos_bottom_center,
        pos_mid_left,
        pos_mid_right,
    ];

    let uvs = vec![
        uv_bottom_left,
        uv_top_center,
        uv_bottom_right,
        uv_bottom_center,
        uv_mid_left,
        uv_mid_right,
    ];

    // All normals point forward
    let normals = vec![[0.0, 0.0, 1.0]; 6];

    // Indices for the 4 new triangles (CCW)
    let indices = Indices::U32(vec![
        0, 3, 4, // Bottom-left triangle
        3, 2, 5, // Bottom-right triangle
        4, 5, 1, // Top triangle
        3, 5, 4, // Center triangle
    ]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    // Generate tangents, which are required by the shader for normal mapping and the curve effect.
    mesh.generate_tangents().unwrap();
    mesh.insert_indices(indices);
    mesh
}
