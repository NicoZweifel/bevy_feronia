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
            .add_message::<ScatterChunk<T>>()
            .add_message::<ScatterResults<T>>()
            .add_message::<ScatterFinished<T>>()
            .add_observer(on_add_scatter_root::<T>)
            .add_observer(on_add_scatter_layer::<T>)
            .add_observer(on_chunk_add::<T>)
            .add_systems(
                PreUpdate,
                process_scatter_queue::<T>.in_set(ScatterSet::Ready),
            )
            .add_systems(
                Update,
                (
                    manage_asset_lifecycle::<T>,
                    chunk_init_scatter::<T>.in_set(ChunkSet::Ready),
                    (
                        handle_finished_scatter_tasks::<T>,
                        spawn::<T>.run_if(resource_exists::<Assets<ScatterAsset<T>>>),
                    )
                        .in_set(ScatterSet::Ready),
                ),
            )
            .add_systems(
                FixedPostUpdate,
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
