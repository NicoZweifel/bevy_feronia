use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn init<TIn: Material, TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone>(
    trigger: On<Add, Chunk>,
    mut cmd: Commands,
) {
    let chunk = trigger.target();

    debug!("Chunk added: {chunk}.");

    cmd.entity(chunk)
        .insert(ChunkInitialize::<TIn, TOut>::default());
}

pub fn chunk_init<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut cmd: Commands,
    q_chunks: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitialize<TIn, TOut>>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
    q_root: Query<&ScatterRoot>,
) {
    for (chunk, root_chunk) in q_chunks.iter() {
        debug!("Chunk added: {chunk}.");

        let Ok(layers) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        cmd.entity(chunk).observe(scatter_chunk::<TIn, TOut>);

        for scatter_layer in layers.iter() {
            let Ok(scatter_layer) = q_layer.get(scatter_layer) else {
                continue;
            };

            cmd.trigger_targets(ScatterChunk::<TIn, TOut>::new(scatter_layer), [chunk])
        }

        cmd.entity(chunk).remove::<ChunkInitialize<TIn, TOut>>();
    }
}
