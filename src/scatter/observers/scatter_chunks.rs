use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;

type LayerQueryItem<'a> = (
    Entity,
    &'a ScatterLayerOf,
    Option<&'a Name>,
    Option<&'a ScatterLayerEnabled>,
);

pub fn scatter_chunks<TOut: ScatterMaterial<TIn> + Asset + Clone, TIn: Material>(
    trigger: On<Scatter<TOut, TIn>>,
    mut cmd: Commands,
    q_root: Query<&ChunkRoot>,
    q_layer: Query<LayerQueryItem, (With<ScatterLayer>, With<ScatterLayerType<TOut, TIn>>)>,
) {
    let Ok((layer_entity, scatter_root, layer_name, enabled)) = q_layer.get(trigger.entity) else {
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok(child_chunks) = q_root.get(**scatter_root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    if !scatter_layer_enabled(&mut cmd, layer_entity, layer_name, enabled) {
        return;
    };

    child_chunks
        .iter()
        .map(|x| ScatterChunk::<TOut, TIn>::new(x, layer_entity))
        .for_each(|x| cmd.trigger(x));
}
