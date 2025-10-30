#[path = "utils/example.rs"]
mod example;

use bevy::camera::primitives::Aabb;
use bevy::color::palettes::tailwind::*;
use bevy::mesh::PlaneMeshBuilder;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
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
            ExtendedWindAffectedPlugin,
        ))
        // Need to wait for the wind noise texture.
        .add_systems(Startup, setup.after(setup_wind_texture))
        .run()
}

fn setup(
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut extended_materials: ResMut<Assets<ExtendedWindAffectedMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    wind: Res<Wind>,
    noise_texture: Res<WindTexture>,
) {
    let mesh = meshes.add(Cuboid::new(0.5, 3.0, 0.5));

    cmd.spawn((
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GRAY_500.into(),
            ..default()
        })),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
        children![(
            MeshMaterial3d(extended_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: GREEN_500.into(),
                    ..default()
                },
                extension: WindAffectedExtension {
                    wind: *wind,
                    aabb: Aabb {
                        half_extents: Vec3A::new(0.5, 3., 0.5),
                        center: Vec3A::new(0.0, 0.0, 0.0),
                    },
                    options: MaterialOptions {
                        // make it affected by wind
                        wind_affected: true,
                        // can also tweak other settings here
                        ..default()
                    },
                    noise_texture: (**noise_texture).clone(),
                },
            })),
            Mesh3d(mesh.clone()),
        )],
    ));
}
