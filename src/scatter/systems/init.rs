use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy_ecs::prelude::*;
#[cfg(feature = "trace")]
use tracing::debug;

pub fn on_chunk_add<T>(trigger: On<Add, ChunkInitialize>, mut cmd: Commands)
where
    T: ScatterMaterial,
{
    let chunk = trigger.entity;

    cmd.entity(chunk).insert(ChunkInitScatter::<T>::default());
}

pub fn chunk_init_scatter<T>(
    mut cmd: Commands,
    q_chunks: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitScatter<T>>, Without<Merging>)>,
    q_layer: Query<
        Entity,
        (
            With<ScatterLayer>,
            With<ScatterLayerType<T>>,
            With<ScatterChunked>,
        ),
    >,
    q_root: Query<&ScatterRoot>,
) where
    T: ScatterMaterial,
{
    for (chunk, root_chunk) in q_chunks.iter() {
        let Ok(layers) = q_root.get(**root_chunk) else {
            #[cfg(feature = "trace")]
            debug!("Couldn't get ScatterRoot: {}", **root_chunk);
            return;
        };

        cmd.entity(chunk).observe(scatter_chunk::<T>);

        for scatter_layer in layers.iter().filter_map(|e| q_layer.get(e).ok()) {
            cmd.trigger(ScatterChunk::<T>::new(chunk, scatter_layer))
        }

        cmd.entity(chunk)
            .remove::<(ChunkInitScatter<T>, ChunkInitialize)>();
    }
}
