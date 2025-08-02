use crate::prelude::*;
use bevy::prelude::*;

pub fn on_split_chunk(
    mut er_split: EventReader<SplitChunk>,
    mut cmd: Commands,
    q_chunk: Query<&ChunkOf, With<Chunk>>,
    q_root: Query<&ScatterRoot>,
) {
    for e in er_split.read() {
        let Ok(root_chunk) = q_chunk.get(**e) else {
            warn!("Couldn't get Chunk: {}", **e);
            return;
        };

        let Ok(root) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **e);
            return;
        };

        for scatter_layer in root.iter() {
            cmd.trigger_targets(ScatterChunk { scatter_layer }, [**e])
        }
    }
}
