use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn on_chunk_add<TOut, TIn>(trigger: On<Add, Chunk>, mut cmd: Commands)
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    let chunk = trigger.entity;

    cmd.entity(chunk)
        .insert(ChunkInitScatter::<TOut, TIn>::default());
}

pub fn chunk_init_scatter<TOut, TIn>(
    mut cmd: Commands,
    q_chunks: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitScatter<TOut, TIn>>)>,
    q_layer: Query<
        Entity,
        (
            With<ScatterLayer>,
            With<ScatterLayerType<TOut, TIn>>,
            With<ScatterChunked>,
        ),
    >,
    q_root: Query<&ScatterRoot>,
) where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    for (chunk, root_chunk) in q_chunks.iter() {
        let Ok(layers) = q_root.get(**root_chunk) else {
            warn!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        cmd.entity(chunk).observe(scatter_chunk::<TOut, TIn>);

        for scatter_layer in layers.iter() {
            let Ok(scatter_layer) = q_layer.get(scatter_layer) else {
                continue;
            };

            cmd.trigger(ScatterChunk::<TOut, TIn>::new(chunk, scatter_layer))
        }

        cmd.entity(chunk).remove::<ChunkInitScatter<TOut, TIn>>();
    }
}
