use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use crate::scatter::utils::scatter_layer_disabled;
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

type LayerQueryItem = (Entity, Has<ScatterLayerDisabled>, Option<&'static Name>);

type LayerQueryFilter<T> = (
    With<ScatterLayer>,
    With<ScatterLayerType<T>>,
    With<ScatterChunked>,
);

pub fn chunk_init_scatter<T>(
    mut cmd: Commands,
    q_chunks: Query<(Entity, &ChunkOf), (With<Chunk>, With<ChunkInitScatter<T>>, Without<Merging>)>,
    q_layer: Query<LayerQueryItem, LayerQueryFilter<T>>,
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

        for scatter_layer in layers
            .iter()
            .filter_map(|e| q_layer.get(e).ok())
            .filter_map(|(e, enabled, name)| !scatter_layer_disabled(e, name, enabled).then_some(e))
        {
            cmd.trigger(ScatterChunk::<T>::new(chunk, scatter_layer))
        }

        cmd.entity(chunk)
            .remove::<(ChunkInitScatter<T>, ChunkInitialize)>();
    }
}
