use crate::prelude::*;
use bevy::prelude::*;

pub fn init<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone>(
    mut cmd: Commands,
    q_chunk: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitialize>)>,
    q_root: Query<&ScatterRoot>,
) {
    for (chunk, root_chunk) in &q_chunk {
        let Ok(layers) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        for scatter_layer in layers.iter() {
            cmd.trigger_targets(ScatterChunk::<TIn, TOut>::new(scatter_layer), [chunk])
        }

        cmd.entity(chunk).remove::<ChunkInitialize>();
    }
}
