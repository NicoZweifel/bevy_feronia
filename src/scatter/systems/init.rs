use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn init<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone>(
    mut cmd: Commands,
    q_chunk: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitialize>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
    q_root: Query<&ScatterRoot>,
) {
    for (chunk, root_chunk) in &q_chunk {
        let Ok(layers) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        cmd.entity(chunk)
            .insert(ScatterObserver)
            .observe(scatter_chunk::<TIn, TOut>);

        for scatter_layer in layers.iter() {
            let Ok(scatter_layer) = q_layer.get(scatter_layer) else {
                continue;
            };

            cmd.trigger_targets(ScatterChunk::<TIn, TOut>::new(scatter_layer), [chunk])
        }

        cmd.entity(chunk).remove::<ChunkInitialize>();
    }
}
