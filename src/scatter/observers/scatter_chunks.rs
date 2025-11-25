use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_ecs::prelude::*;

#[cfg(feature = "tracing")]
use tracing::warn;

type LayerQueryItem = (
    Entity,
    &'static ScatterLayerOf,
    Option<&'static Name>,
    Option<&'static ScatterLayerEnabled>,
);

pub fn scatter_chunks<T: ScatterMaterial>(
    trigger: On<Scatter<T>>,
    mut cmd: Commands,
    q_root: Query<&ChunkRoot>,
    q_layer: Query<LayerQueryItem, (With<ScatterLayer>, With<ScatterLayerType<T>>)>,
) {
    let Ok((layer_entity, scatter_root, layer_name, enabled)) = q_layer.get(trigger.entity) else {
        #[cfg(feature = "tracing")]
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok(child_chunks) = q_root.get(**scatter_root) else {
        #[cfg(feature = "tracing")]
        warn!("ScatterRoot not found!");
        return;
    };

    if !scatter_layer_enabled(&mut cmd, layer_entity, layer_name, enabled) {
        return;
    };

    child_chunks
        .iter()
        .map(|c| ScatterChunk::<T>::new(c, layer_entity))
        .for_each(|c| cmd.trigger(c));
}
