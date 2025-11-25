use crate::prelude::*;
use crate::scatter::observers::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;

#[cfg(feature = "tracing")]
use tracing::{debug, warn};

pub fn on_add_scatter_layer<T>(
    trigger: On<Add, ScatterLayerType<T>>,
    mut cmd: Commands,
    layer_query: Query<
        (&ChildOf, Option<&ScatterChunked>),
        (
            With<ScatterLayer>,
            With<ScatterLayerType<T>>,
            Without<ScatterObserver>,
        ),
    >,
    root_query: Query<Option<&ChunkRoot>, With<ScatterRoot>>,
) where
    T: ScatterMaterial,
{
    let layer = trigger.entity;

    #[cfg(feature = "tracing")]
    debug!("Added ScatterLayer {layer}.");

    let Ok((layer_root, scatter_chunked)) = layer_query.get(layer) else {
        #[cfg(feature = "tracing")]
        warn!("Could not get ScatterLayer {layer}!");
        return;
    };

    cmd.entity(layer)
        .insert((ScatterObserver, ScatterLayerOf(layer_root.get())));

    let chunk_root = root_query.get(layer_root.get()).unwrap();
    if chunk_root.is_some() && scatter_chunked.is_some() {
        cmd.entity(layer).observe(scatter_chunks::<T>);
    } else {
        cmd.entity(layer).observe(scatter::<T>);
    }
}
