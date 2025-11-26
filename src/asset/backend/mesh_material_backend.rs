use crate::asset::backend::iter_self_and_descendants_with_component::iter_self_and_descendants_with_component;
use crate::asset::backend::systems::backend;
use crate::backend::ScatterApp;
use crate::prelude::{AssetItem, AssetItemOf};
use crate::prelude::{NeedsAssetCollection, ScatterLayer};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{MeshMaterial3d, StandardMaterial};

#[cfg(feature = "tracing")]
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
    let scene_entity = trigger.entity;

    cmd.entity(scene_entity)
        .insert(NeedsAssetCollection::default());
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
) -> Result<Vec<AssetItem>> {
    Ok(q_collect
        .iter()
        .filter_map(|layer| q_children.get(layer).ok())
        .flatten()
        .filter_map(|root| {
            let child_of = q_parent
                .get(*root)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!("Could not get parent!");
                })
                .ok()?;
            let layer = child_of.parent();
            let (layer, _name) = q_layers
                .get(layer)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    warn!("Could not get ScatterLayer!");
                })
                .ok()?;

            #[cfg(feature = "tracing")]
            debug!(
                "Collecting assets in {} {layer}...",
                _name.cloned().unwrap_or_default()
            );

            Some((root, layer))
        })
        .filter_map(|(root, layer)| {
            Some(
                q_children
                    .get(*root)
                    .map_err(|_| {
                        #[cfg(feature = "tracing")]
                        warn!("Could not get children of root!");
                    })
                    .ok()?
                    .iter()
                    .flat_map(|item_root| {
                        iter_self_and_descendants_with_component(item_root, &q_children, &q_search)
                            .into_iter()
                            .map(move |item| (item_root, item))
                    })
                    .map(move |(root_item, child)| {
                        AssetItem::new(
                            child,
                            AssetItemOf::new(root_item, *root, layer)
                                .with_name_from_queries(child, &q_name, &q_parent),
                        )
                    }),
            )
        })
        .flatten()
        .collect())
}
