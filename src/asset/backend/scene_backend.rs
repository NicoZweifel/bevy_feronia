use crate::asset::backend::prelude::*;
use crate::prelude::AssetPart;
use crate::prelude::ScatterLayer;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_scene::{SceneInstanceReady, SceneRoot};

use crate::asset::backend::iter_self_and_descendants_with_component;
use crate::asset::backend::systems::backend;
use crate::backend::ScatterApp;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub struct SceneAssetBackendPlugin;

impl Plugin for SceneAssetBackendPlugin {
    fn build(&self, app: &mut App) {
        app.set_scatter_asset_backend(scene_asset_backend)
            .add_observer(scene_asset_ready_listener)
            .add_systems(PostUpdate, backend);
    }
}

/// A lightweight listener that tags a scene root as ready for processing
/// once the SceneInstanceReady event fires.
pub fn scene_asset_ready_listener(
    trigger: On<SceneInstanceReady>,
    mut cmd: Commands,
    q_data: Query<(), (With<Children>, With<ChildOf>)>,
) {
    let scene_entity = trigger.entity;

    if q_data.get(scene_entity).is_err() {
        #[cfg(feature = "trace")]
        debug!(
            "Scene asset {:?} is not a processable scatter asset, skipping.",
            scene_entity
        );
        return;
    };

    cmd.entity(scene_entity)
        .insert(NeedsAssetCollection::default());
}

/// A `ScatterAsset` Backend system that collects [`Mesh3d`]/[`MeshMaterial3d`] combinations recursively in a Scene.
///
/// The `SceneRoot` has a root collection which is always assumed to contain parents of all assets in the Tree, e.g.,
/// all [`ScatterAssetPart`]s will be assigned to the children of the root collection.
pub fn scene_asset_backend(
    _: In<()>,
    q_collect: Query<Entity, (With<SceneRoot>, With<NeedsAssetCollection>)>,
    q_layers: Query<(Entity, Option<&Name>), With<ScatterLayer>>,
    q_parent: Query<&ChildOf>,
    q_children: Query<&Children>,
    q_search: Query<Entity, (With<Mesh3d>, With<MeshMaterial3d<StandardMaterial>>)>,
    q_name: Query<&Name>,
) -> Result<Vec<AssetPart>> {
    Ok(q_collect
        .iter()
        .filter_map(|scene_root| {
            let child_of = q_parent
                .get(scene_root)
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

            Some((scene_root, layer))
        })
        .filter_map(|(scene_root, layer)| {
            Some(q_children
                .get(scene_root).inspect_err(|_| {
                #[cfg(feature = "trace")]
                warn!("Could not get children of scene root!");
            }).ok()?
                .iter()
                // We only want to collect assets that are children of the root collection if we use this backend
                // since gltf scenes have a root collection.
                .filter_map(|root_collection| Some(q_children.get(root_collection).inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!("Could not get children of root collection!");
                }).ok()?.iter()))
                .flatten()
                .flat_map(|item_root| {
                    iter_self_and_descendants_with_component::iter_self_and_descendants_with_component(
                        item_root,
                        &q_children,
                        &q_search,
                    )
                        .into_iter()
                        .map(move |item| (item_root, item))
                })
                .map(move |(root_item, child)| {
                    AssetPart::new(
                        child,
                        AssetPartOf::new(root_item, scene_root, layer)
                            .with_name_from_queries(child, &q_name, &q_parent),
                    )
                }))
        })
        .flatten()
        .collect())
}
