use crate::prelude::*;
use crate::scatter::observers::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

pub fn on_add_scatter_layer<TOut, TIn>(
    trigger: On<Add, ScatterLayerType<TOut, TIn>>,
    mut cmd: Commands,
    layer_query: Query<
        (&ChildOf, Option<&ScatterChunked>),
        (
            With<ScatterLayer>,
            With<ScatterLayerType<TOut, TIn>>,
            Without<ScatterObserver>,
        ),
    >,
    root_query: Query<Option<&ChunkRoot>, With<ScatterRoot>>,
) where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    let layer = trigger.entity;

    debug!("Added ScatterLayer {layer}.");

    let Ok((layer_root, scatter_chunked)) = layer_query.get(layer) else {
        warn!("Could not get ScatterLayer {layer}!");
        return;
    };

    cmd.entity(layer)
        .insert((ScatterObserver, ScatterLayerOf(layer_root.get())));

    let chunk_root = root_query.get(layer_root.get()).unwrap();
    if chunk_root.is_some() && scatter_chunked.is_some() {
        cmd.entity(layer).observe(scatter_chunks::<TOut, TIn>);
    } else {
        cmd.entity(layer).observe(scatter::<TOut, TIn>);
    }
}
