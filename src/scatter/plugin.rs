use crate::asset::backend::systems::{insert_parts, insert_requests};
use crate::asset::systems::*;
use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use crate::scatter::{observers::*, systems::prelude::*};

use bevy_app::prelude::*;
use bevy_asset::prelude::{AssetApp, Assets};
use bevy_ecs::prelude::resource_exists;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_pbr::prelude::StandardMaterial;
use bevy_state::prelude::*;

use bevy_transform::TransformSystems;
use std::marker::PhantomData;

pub struct ScatterAssetPlugin<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    _phantom: PhantomData<T>,
}

impl<T> ScatterAssetPlugin<T>
where
    T: ScatterMaterial,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
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
            .add_message::<Scatter<T>>()
            .init_asset::<ScatterAsset<T>>()
            .init_asset::<ScatterAsset<T>>()
            .add_message::<ScatterChunk<T>>()
            .add_message::<ScatterResults<T>>()
            .add_message::<ScatterFinished<T>>()
            .add_observer(on_add_scatter_root::<T>)
            .add_observer(on_add_scatter_layer::<T>)
            .add_observer(on_chunk_add::<T>)
            .add_systems(Update, chunk_init_scatter::<T>.in_set(ChunkSet::Ready))
            .add_systems(
                PreUpdate,
                process_scatter_queue::<T>.run_if(in_state(ScatterState::Ready)),
            )
            .add_systems(
                Update,
                (
                    handle_scatter_requests::<T>,
                    handle_finished_scatter_tasks::<T>,
                    spawn::<T>.run_if(resource_exists::<Assets<ScatterAsset<T>>>),
                )
                    .run_if(in_state(ScatterState::Ready)),
            );
    }
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ChunkPlugin, HeightMapPlugin, WindPlugin))
            .init_state::<ScatterState>()
            .configure_sets(
                Update,
                (
                    ScatterSet::Loading.run_if(in_state(ScatterState::Loading)),
                    ScatterSet::Setup.run_if(in_state(ScatterState::Setup)),
                    ScatterSet::Collecting.run_if(in_state(ScatterState::Collecting)),
                    ScatterSet::Ready.run_if(in_state(ScatterState::Ready)),
                ),
            )
            .configure_sets(
                PostUpdate,
                (
                    ScatterSet::Loading.run_if(in_state(ScatterState::Loading)),
                    ScatterSet::Setup.run_if(in_state(ScatterState::Setup)),
                    ScatterSet::Collecting.run_if(in_state(ScatterState::Collecting)),
                    ScatterSet::Ready.run_if(in_state(ScatterState::Ready)),
                ),
            )
            .init_resource::<WorldSeed>()
            .add_message::<ClearScatterLayer>()
            .add_message::<ClearScatterRoot>()
            .add_observer(on_add_scatter_item)
            .add_systems(
                PostUpdate,
                setup_root_aabb
                    .in_set(ScatterSet::Setup)
                    .after(TransformSystems::Propagate),
            )
            .add_systems(
                Update,
                (transition_to_collecting,).in_set(ScatterSet::Setup),
            )
            .add_systems(
                Update,
                (
                    check_unprocessed_layers,
                    check_unprocessed_items,
                    check_unprocessed_root,
                )
                    .in_set(ScatterSet::Collecting),
            )
            .add_systems(
                Update,
                (
                    clear_scatter_roots,
                    (clear_chunks, clear_scatter_layers).after(clear_scatter_roots),
                )
                    .in_set(ScatterSet::Ready),
            );
    }
}

pub struct StandardScatterPlugin;

impl Plugin for StandardScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnScatterAssets>()
            .add_message::<SpawnScatterAssets>()
            .add_plugins(ScatterMaterialPlugin::<StandardMaterial>::new())
            .add_plugins(ScatterAssetPlugin::<StandardMaterial>::new())
            .add_systems(
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
