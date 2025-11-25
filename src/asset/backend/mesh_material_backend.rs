use crate::prelude::ScatterLayer;
use crate::prelude::{AssetItem, AssetItemOf};

use bevy_ecs::prelude::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{MeshMaterial3d, StandardMaterial};

use crate::asset::backend::iter_self_and_descendants_with_component::iter_self_and_descendants_with_component;
#[cfg(feature = "tracing")]
use tracing::debug;

/// A `ScatterAsset` Backend system that collects [`Mesh3d`]/[`MeshMaterial3d`] combinations recursively.
pub fn mesh_material_backend(
    scene_root: In<Entity>,
    q_layers: Query<(Entity, Option<&Name>), With<ScatterLayer>>,
    q_name: &Query<&Name>,
    q_parent: &Query<&ChildOf>,
    q_children: Query<&Children>,
    q_search: Query<Entity, (With<Mesh3d>, With<MeshMaterial3d<StandardMaterial>>)>,
) -> Result<Vec<AssetItem>> {
    let scene_root = *scene_root;
    let child_of = q_parent.get(scene_root)?;
    let layer = child_of.parent();
    let (layer, _name) = q_layers.get(layer)?;

    #[cfg(feature = "tracing")]
    debug!(
        "Collecting assets in {:?} {layer}...",
        _name.cloned().unwrap_or_default()
    );

    Ok(q_children
        .get(scene_root)?
        .iter()
        .flat_map(|item_root| {
            iter_self_and_descendants_with_component(item_root, &q_children, &q_search)
                .into_iter()
                .map(move |item| (item_root, item))
        })
        .map(|(root_item, child)| {
            AssetItem::new(
                child,
                AssetItemOf::new(root_item, scene_root, layer)
                    .with_name_from_queries(child, &q_name, &q_parent),
            )
        })
        .collect())
}
