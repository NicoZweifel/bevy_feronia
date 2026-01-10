use crate::backend::ScatterApp;
use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use crate::scatter::{observers::*, systems::prelude::*};
use crate::{
    asset::backend::systems::{insert_parts, insert_requests},
    asset::systems::*,
};

use bevy_app::prelude::*;
use bevy_asset::prelude::{AssetApp, Assets};
use bevy_ecs::{prelude::resource_exists, schedule::IntoScheduleConfigs};
use bevy_pbr::prelude::StandardMaterial;
use bevy_state::prelude::*;

use bevy_transform::TransformSystems;
use std::marker::PhantomData;

pub struct ScatterAssetPlugin<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    _marker: PhantomData<T>,
}

impl<T> ScatterAssetPlugin<T>
where
    T: ScatterMaterial,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for ScatterAssetPlugin<T>
where
    T: ScatterMaterial,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Plugin for ScatterAssetPlugin<T>
where
    T: ScatterMaterial,
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ScatterPlugin>() {
            app.add_plugins(ScatterPlugin);
        }

        app.init_resource::<SpawnScatterAssetsEventQueue<T>>()
            .init_resource::<ScatterAssetManager<T>>()
            .init_asset::<ScatterAsset<T>>()
            .add_message::<Scatter<T>>()
            .add_message::<ScatterChunk<T>>()
            .add_message::<ScatterResults<T>>()
            .add_message::<ScatterFinished<T>>()
            .add_observer(on_add_scatter_root::<T>)
            .add_observer(on_add_scatter_layer::<T>)
            .add_observer(on_chunk_add::<T>)
            .add_systems(Update, chunk_init_scatter::<T>.in_set(ChunkSet::Ready))
            .add_systems(
                PreUpdate,
                process_scatter_queue::<T>.in_set(ScatterSet::Ready),
            )
            .add_systems(
                Update,
                (
                    handle_finished_scatter_tasks::<T>,
                    spawn::<T>.run_if(resource_exists::<Assets<ScatterAsset<T>>>),
                )
                    .in_set(ScatterSet::Ready),
            )
            .add_systems(
                PostUpdate,
                handle_scatter_requests::<T>
                    .after(TransformSystems::Propagate)
                    .in_set(ScatterSet::Ready),
            );
    }
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.configure_scatter_sets()
            .add_plugins((
                ChunkPlugin,
                HeightMapPlugin,
                WindPlugin,
                ScatterOccupancyMapPlugin,
            ))
            .init_state::<ScatterState>()
            .init_resource::<WorldSeed>()
            .add_message::<ClearScatterLayer>()
            .add_message::<ClearScatterRoot>()
            .add_observer(on_add_scatter_item)
            .add_systems(
                Update,
                (
                    transition_to_collecting.in_set(ScatterSet::Setup),
                    (
                        check_unprocessed_layers,
                        check_unprocessed_items,
                        check_unprocessed_root,
                    )
                        .in_set(ScatterSet::Collecting),
                    clear_scatter_roots.in_set(ScatterSet::Ready),
                    (clear_chunks, clear_scatter_layers).in_set(ScatterSet::Clean),
                ),
            )
            .add_systems(
                PostUpdate,
                setup_root_aabb
                    .in_set(ScatterSet::Setup)
                    .after(TransformSystems::Propagate),
            );
    }
}

pub struct StandardScatterPlugin;

impl Plugin for StandardScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnScatterAssets>()
            .add_message::<SpawnScatterAssets>()
            .add_plugins((
                ScatterMaterialPlugin::<StandardMaterial>::new(),
                StandardScatterAssetRequestPlugin,
                ScatterAssetPlugin::<StandardMaterial>::new(),
            ));
    }
}

pub struct StandardScatterAssetRequestPlugin;

impl Plugin for StandardScatterAssetRequestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                insert_parts::<StandardMaterial>,
                insert_requests::<StandardMaterial>,
            )
                .in_set(ScatterSet::Collecting),
        )
        .add_systems(
            Update,
            process_standard_requests.in_set(ScatterSet::Collecting),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy_asset::AssetEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            MaterialPlugin::<StandardMaterial>::default(),
        ))
        .init_asset::<ScatterAsset<StandardMaterial>>()
        .init_resource::<ScatterAssetManager<StandardMaterial>>();

        app
    }

    #[test]
    fn test_process_standard_requests_should_register_asset_and_clean_up_entity() {
        // Arrange
        let mut app = setup_app();
        app.add_systems(Update, process_standard_requests);

        let mesh_handle = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            Default::default(),
        ));
        let mat_handle = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial::default());
        let layer_entity = app.world_mut().spawn_empty().id();
        let request_entity = app.world_mut().spawn_empty().id();

        let part = ScatterAssetPart {
            h_mesh: mesh_handle,
            h_material: mat_handle,
            transform: Transform::default(),
            properties: ScatterAssetProperties::default(),
            name: None,
            #[cfg(feature = "avian")]
            collider: None,
        };

        // Act
        app.world_mut().entity_mut(request_entity).insert(
            ScatterAssetCreationRequest::<StandardMaterial>::new(
                ScatterAssetProperties::default(),
                vec![part],
                layer_entity,
                #[cfg(feature = "avian")]
                None,
            )
        );
        app.update();

        // Assert
        let manager = app.world().resource::<ScatterAssetManager<StandardMaterial>>();
        assert_eq!(manager.asset_to_layer.len(), 1, "Manager should have 1 registered asset");

        let (asset_id, &stored_layer) = manager.asset_to_layer.iter().next().unwrap();
        assert_eq!(stored_layer, layer_entity, "Manager should point to the correct layer entity");

        let scatter_assets = app.world().resource::<Assets<ScatterAsset>>();
        assert!(scatter_assets.contains(*asset_id), "The ScatterAsset should exist in the Assets resource");

        assert!(
            app.world().entity(request_entity).get::<ScatterAssetCreationRequest>().is_none(),
            "The CreationRequest component should be removed after processing"
        );
        assert!(
            app.world().entity(request_entity).contains::<ScatterItem>(),
            "The entity should have received the ScatterItem marker"
        );
    }

    #[test]
    fn test_manage_asset_lifecycle_should_remove_entry() {
        // Arrange
        let mut app = setup_app();
        app.add_systems(Update, manage_asset_lifecycle::<StandardMaterial>);

        let layer_entity = app.world_mut().spawn_empty().id();
        let asset_handle = app.world_mut().resource_mut::<Assets<ScatterAsset>>().add(ScatterAsset::default());
        let asset_id = asset_handle.id();

        app.world_mut()
            .resource_mut::<ScatterAssetManager<StandardMaterial>>()
            .asset_to_layer
            .insert(asset_id, layer_entity);

        // Act
        app.world_mut().write_message(AssetEvent::Removed { id: asset_id });
        app.update();

        // Assert
        let manager = app.world().resource::<ScatterAssetManager<StandardMaterial>>();
        assert!(
            !manager.asset_to_layer.contains_key(&asset_id),
            "The asset ID should be removed from the manager after the AssetEvent::Removed"
        );
    }
}
