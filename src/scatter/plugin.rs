use crate::asset::systems::*;
use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use crate::scatter::observers::*;
use crate::scatter::systems::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct ScatterAssetPlugin<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    _phantom: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> ScatterAssetPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<TOut, TIn> Default for ScatterAssetPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TOut, TIn> Plugin for ScatterAssetPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ScatterPlugin>() {
            app.add_plugins(ScatterPlugin);
        }

        app.add_message::<Scatter<TOut, TIn>>()
            .init_asset::<ScatterAsset<TIn>>()
            .init_asset::<ScatterAsset<TOut>>()
            .add_message::<ScatterChunk<TOut, TIn>>()
            .add_message::<ScatterResults<TOut, TIn>>()
            .add_message::<ScatterFinished<TOut, TIn>>()
            .add_observer(on_add_scatter_root::<TOut, TIn>)
            .add_observer(on_add_scatter_layer::<TOut, TIn>)
            .add_observer(on_chunk_add::<TOut, TIn>)
            .add_observer(scatter_finished::<TOut, TIn>)
            .add_systems(
                Update,
                chunk_init_scatter::<TOut, TIn>.in_set(ChunkSet::Ready),
            )
            .add_systems(
                Update,
                (
                    handle_scatter_requests::<TOut, TIn>,
                    handle_finished_scatter_tasks::<TOut, TIn>,
                    spawn::<TOut, TIn>.run_if(resource_exists::<Assets<ScatterAsset>>),
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
            .init_resource::<WorldSeed>()
            .add_message::<ClearScatterLayer>()
            .add_message::<ClearScatterRoot>()
            .add_observer(on_add_scatter_item)
            .add_systems(
                PostUpdate,
                setup_root_aabb.run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (transition_to_ready_state,).run_if(in_state(ScatterState::Setup)),
            )
            .add_systems(
                Update,
                (
                    check_unprocessed_layers,
                    check_unprocessed_items,
                    clear_scatter_roots,
                    clear_scatter_layers.after(clear_scatter_roots),
                )
                    .run_if(in_state(ScatterState::Ready)),
            );
    }
}

pub struct StandardScatterPlugin;

impl Plugin for StandardScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnScatterAssets<StandardMaterial>>()
            .add_message::<SpawnScatterAssets<StandardMaterial>>()
            .add_plugins(ScatterMaterialPlugin::<StandardMaterial>::default())
            .add_plugins((ScatterAssetPlugin::<StandardMaterial>::new(),))
            .add_systems(
                Update,
                (
                    queue_material_creation_requests::<StandardMaterial, StandardMaterial>,
                    process_same_type_material_requests::<StandardMaterial>.after(
                        queue_material_creation_requests::<StandardMaterial, StandardMaterial>,
                    ),
                )
                    .run_if(in_state(ScatterState::Ready)),
            );
    }
}
