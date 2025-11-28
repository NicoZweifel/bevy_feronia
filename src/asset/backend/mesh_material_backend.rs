use crate::asset::backend::iter_self_and_descendants_with_component::iter_self_and_descendants_with_component;
use crate::asset::backend::systems::backend;
use crate::backend::ScatterApp;
use crate::prelude::{AssetPart, AssetPartOf};
use crate::prelude::{NeedsAssetCollection, ScatterLayer};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{MeshMaterial3d, StandardMaterial};

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub struct MeshMaterialAssetBackendPlugin;

impl Plugin for MeshMaterialAssetBackendPlugin {
    fn build(&self, app: &mut App) {
        app.set_scatter_asset_backend(mesh_material_backend)
            .add_observer(on_add_layer)
            .add_systems(PostUpdate, backend);
    }
}

/// A lightweight listener that tags children of a layer as ready for processing
pub fn on_add_layer(trigger: On<Add, ScatterLayer>, mut cmd: Commands) {
    let layer = trigger.entity;

    cmd.entity(layer).insert(NeedsAssetCollection);
}

/// A `ScatterAsset` Backend system that collects [`Mesh3d`]/[`MeshMaterial3d`] combinations recursively.
pub fn mesh_material_backend(
    _: In<()>,
    q_collect: Query<Entity, With<NeedsAssetCollection>>,
    q_layers: Query<(Entity, Option<&Name>), With<ScatterLayer>>,
    q_parent: Query<&ChildOf>,
    q_children: Query<&Children>,
    q_search: Query<Entity, (With<Mesh3d>, With<MeshMaterial3d<StandardMaterial>>)>,
    q_name: Query<&Name>,
) -> Result<Vec<AssetPart>> {
    Ok(q_collect
        .iter()
        .filter_map(|layer| q_children.get(layer).ok())
        .flatten()
        .filter_map(|root| {
            let child_of = q_parent
                .get(*root)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!("Could not get parent!");
                })
                .ok()?;
            let layer = child_of.parent();
            let (layer, _name) = q_layers
                .get(layer)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!("Could not get ScatterLayer!");
                })
                .ok()?;

            #[cfg(feature = "trace")]
            debug!(
                "Collecting assets in {} {layer}...",
                _name.cloned().unwrap_or_default()
            );

            Some((root, layer))
        })
        .flat_map(|(root, layer)| {
            iter_self_and_descendants_with_component(*root, &q_children, &q_search).map(
                move |child| {
                    AssetPart::new(
                        child,
                        AssetPartOf::new(*root, *root, layer)
                            .with_name_from_queries(child, &q_name, &q_parent),
                    )
                },
            )
        })
        .collect())
}
