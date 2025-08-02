use crate::chunking::systems::prelude::split;
use crate::scatter::observers::*;
use crate::scatter::systems::prelude::*;
use bevy::prelude::*;
use crate::prelude::*;

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Scatter<ScatterRoot>>()
            .add_event::<Scatter<ScatterLayer>>()
            .add_event::<ScatterChunk>()
            .add_event::<ScatterResults>()
            .add_event::<SplitChunk>()
            .add_observer(on_add_chunk)
            .add_observer(on_add_scatter_root)
            .add_observer(on_add_scatter_layer)
            .add_systems(Update, (setup_root, on_split_chunk.after(split)));
    }
}
