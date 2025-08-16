use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;

type LayerQueryItem<'a> = (
    Entity,
    &'a ScatterLayerOf,
    Option<&'a Name>,
    Option<&'a ScatterLayerEnabled>,
);

pub fn scatter_chunks<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut trigger: On<Scatter<TIn, TOut>>,
    mut cmd: Commands,
    q_root: Query<&ChunkRoot>,
    q_layer: Query<LayerQueryItem, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
) {
    trigger.propagate(false);

    let Ok((layer_entity, scatter_root, layer_name, enabled)) = q_layer.get(trigger.target())
    else {
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

    cmd.trigger_targets(
        ScatterChunk::<TIn, TOut>::new(layer_entity),
        child_chunks.iter().collect::<Vec<_>>(),
    );
}
