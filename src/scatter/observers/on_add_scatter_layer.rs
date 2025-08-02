use crate::prelude::*;
use crate::scatter::observers::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

pub fn on_add_scatter_layer(
    trigger: On<Add, ScatterLayer>,
    mut cmd: Commands,
    layer_query: Query<(Entity, &ChildOf), With<ScatterLayer>>,
    root_query: Query<Option<&ChunkRoot>, With<ScatterRoot>>,
) {
    let Ok((layer, root_layer)) = layer_query.get(trigger.target()) else {
        warn!("Could not get ScatterLayer {}", trigger.target());
        return;
    };

    cmd.entity(layer).insert(ScatterLayerOf(root_layer.get()));

    let chunk_root = root_query.get(root_layer.get()).unwrap();
    if chunk_root.is_some() {
        cmd.entity(layer).observe(scatter_chunks);
    } else {
        cmd.entity(layer).observe(scatter);
    }
}
