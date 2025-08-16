use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn on_chunk_add<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    trigger: On<Add, Chunk>,
    mut cmd: Commands,
) {
    let chunk = trigger.target();

    cmd.entity(chunk)
        .insert(ChunkInitScatter::<TIn, TOut>::default());
}

pub fn chunk_init_scatter<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut cmd: Commands,
    q_chunks: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitScatter<TIn, TOut>>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
    q_root: Query<&ScatterRoot>,
) {
    for (chunk, root_chunk) in q_chunks.iter() {
        let Ok(layers) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        debug!("Chunk initial scatter: {chunk}.");

        cmd.entity(chunk).observe(scatter_chunk::<TIn, TOut>);

        for scatter_layer in layers.iter() {
            let Ok(scatter_layer) = q_layer.get(scatter_layer) else {
                continue;
            };

            cmd.trigger_targets(ScatterChunk::<TIn, TOut>::new(scatter_layer), [chunk])
        }

        cmd.entity(chunk).remove::<ChunkInitScatter<TIn, TOut>>();
    }
}
