use crate::prelude::*;
use crate::scatter::observers::*;
use crate::scatter::systems::prelude::*;
use bevy::prelude::*;

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Scatter>()
            .init_state::<ScatterState>()
            .init_asset::<ScatterAsset<StandardMaterial>>()
            .init_asset::<ScatterAsset<ExtendedWindAffectedMaterial>>()
            .init_asset::<ScatterAsset<InstancedWindAffectedMaterial>>()
            .add_event::<ScatterChunk>()
            .add_event::<ScatterResults>()
            .add_event::<SplitChunk>()
            .add_observer(on_add_scatter_root)
            .add_observer(on_add_scatter_layer)
            .add_observer(on_add_scatter_item)
            .add_systems(Update, setup_root);
    }
}
