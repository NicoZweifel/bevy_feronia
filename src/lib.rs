//! # bevy_feronia ![crates.io](https://img.shields.io/crates/v/bevy_feronia.svg)
//!
//! Environment scattering tools and shaders/materials that prioritize visual fidelity/artistic freedom, a declarative API and modularity.
//!
//! ## Who is this for?
//!
//! In the current stage this is mostly for tinkerers and learners within the [bevy](https://github.com/bevyengine/bevy) ecosystem, but I am planning to use this for actual game dev myself eventually.
//!
//! > [!CAUTION]
//! > This package is in early development and in an experimentation stage.
//! > I wouldn't personally use this in production quite yet, but it's getting closer to that state incrementally.
//!
//! ## Getting started
//!
//! ```shell
//! cargo add bevy_feronia
//! ```
//!
//! The possible use-cases are demonstrated in the [examples](/examples/EXAMPLES.md)
//!
//! ### Setup
//!
//! The setup depends on the use-case, but a typical setup would look like something like this:
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy_feronia::prelude::*;
//! # let mut app = App::new();
//! app.add_plugins((
//!     MeshMaterialAssetBackendPlugin,
//!     // Or
//!     SceneAssetBackendPlugin,
//!     // ...
//!     ExtendedWindAffectedScatterPlugin
//! ));
//! ```
//!
//! The Scatter system needs to know when it can set up since it can depend on height mapping. You need to insert the setup state at some point.
//!
//! > [!NOTE]
//! > In complex setups that load assets and bake a height map, this can be after the `Startup`.
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy_feronia::prelude::*;
//! # let mut app = App::new();
//! app.insert_state(ScatterState::Setup);
//! ```
//!
//! Or
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy_feronia::prelude::*;
//! # fn system(mut ns_height_map: ResMut<NextState<HeightMapState>>, mut ns_scatter: ResMut<NextState<ScatterState>>) {
//! ns_height_map.set(HeightMapState::Setup);
//! ns_scatter.set(ScatterState::Setup);
//! # }
//! ```
//!
//! ### Defining layers
//!
//! A `ScatterItem`'s `LOD`s are grouped by `Name`. If the names end in `LOD_1` or `lod1` etc., the LOD suffix will be stripped from the name to match it to the other lods of the asset.
//!
//! > [!CAUTION]
//! > When defining multiple `ScatterItems` per `ScatterLayer` without names, a different asset will render when `LODs` are changing, leading to visual bugs.
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy::color::palettes::tailwind::{GRAY_500, RED_500};
//! # use bevy_feronia::prelude::*;
//! # fn setup(
//! #    mut cmd: Commands,
//! #    mut materials: ResMut<Assets<StandardMaterial>>,
//! #    mut meshes: ResMut<Assets<Mesh>>,
//! #    mesh: Handle<Mesh>
//! # ) {
//! // Landscape
//! cmd.spawn((
//!     MeshMaterial3d(materials.add(StandardMaterial {
//!     base_color: GRAY_500.into(),
//!         ..default()
//!     })),
//!     Mesh3d(meshes.add(PlaneMeshBuilder::from_length(80.).build())),
//!     ScatterRoot::default(),
//!     // Scatter layers
//!     children![(
//!             // Make sure you use the correct `ScatterLayer` with the desired `ScatterLayerType`, e.g.,
//!             // Standard, Extended or Instanced Material/Layer.
//!             extension::scatter_layer("Wind Affected Layer"),
//!             // Scatter Options
//!             DistributionDensity(50.),
//!             InstanceJitter::default(),
//!             // You can define material options on the full layer here
//!             WindAffected,
//!             children![
//!                 (
//!                     // Or overwrite on the item, e.g.,
//!                     // WindAffected,
//!                     //
//!                     // CAUTION: If you have multiple assets, all lods that belong to each other need to have the same name!
//!                     //
//!                     // You can have multiple assets in each layer; as long as all LODs have the same name, they will be matched correctly.
//!                     Name::new("Wind Affected Example Item"),
//!                     MeshMaterial3d(materials.add(StandardMaterial::default())),
//!                     Mesh3d(mesh.clone()),
//!                 ),
//!                 (
//!                     Name::new("Wind Affected Example Item"),
//!                     // We need to specify the LOD Level if it is not 0 (Highest level)
//!                     LevelOfDetail(1),
//!                     MeshMaterial3d(materials.add(StandardMaterial {
//!                         base_color: RED_500.into(),
//!                         ..default()
//!                     })),
//!                     Mesh3d(mesh.clone()),
//!                 ),
//!
//!             ]
//!         )]
//! ));
//! # }
//! ```
//!
//! ### Scattering
//!
//! Now you can start scattering! 🌱 🍃 🌿 🍀 🌳 🌲 🌴 🌺
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy_feronia::prelude::*;
//! # fn system(mut cmd: Commands, root: Single<Entity, With<ScatterRoot>>) {
//! cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
//! # }
//! ```
//!
//! > [!NOTE]
//! > `ScatterLayers` and their `ScatterItems` of the same `ScatterType` are always scattered in order, but layers of different `ScatterTypes` can be scattered at the same time.
//!
//! #### Ordered Scattering
//!
//! In complex scenes it is often required to scatter a complete hierarchy in order (rocks → trees/foliage → grass).
//!
//! > [!TIP]
//! > If an ordered scatter is still required, and you can't or don't want to scatter in parallel, observers need to be used to chain the scattering of `ScatterTypes` in order.
//!
//! ```rust,ignore
//! # use bevy::prelude::*;
//! # use bevy_feronia::prelude::*;
//!
//! fn scatter_on_keypress(
//!     mut cmd: Commands,
//!     keyboard_input: Res<ButtonInput<KeyCode>>,
//!     root: Single<Entity, With<ScatterRoot>>
//! ) {
//!     if !keyboard_input.just_pressed(KeyCode::Space) {
//!         return;
//!     };
//!
//!     // Scatter the rocks.
//!     cmd.trigger(Scatter::<StandardMaterial>::new(*root));
//! }
//!
//! fn scatter_extended(
//!     _: Trigger<ScatterFinished<StandardMaterial>>,
//!     mut cmd: Commands,
//!     root: Single<Entity, With<ScatterRoot>>,
//! ) {
//!     // Scatter the foliage after the rocks.
//!     cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
//! }
//!
//! fn scatter_instanced(
//!     _: Trigger<ScatterFinished<ExtendedWindAffectedMaterial>>,
//!     mut cmd: Commands,
//!     root: Single<Entity, With<ScatterRoot>>,
//! ) {
//!     // Scatter the grass last so it doesn't grow on occupied areas.
//!     cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
//! }
//! ```

pub mod asset;
pub mod backend;
pub mod chunking;
pub mod core;
pub mod density_map;
pub mod extension;
pub mod height_map;
pub mod instancing;

pub mod scatter;
pub mod wind;
pub mod prelude {
    pub use crate::asset::prelude::*;
    pub use crate::chunking::prelude::*;
    pub use crate::core::*;
    pub use crate::density_map::*;
    pub use crate::extension::prelude::*;
    pub use crate::height_map::prelude::*;
    pub use crate::instancing::prelude::*;
    pub use crate::scatter::prelude::*;
    pub use crate::wind::prelude::*;
}
