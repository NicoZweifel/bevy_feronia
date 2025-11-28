use crate::prelude::{Chunk, ClearScatterLayer, ScatteredInstance};

use bevy_ecs::prelude::*;
use bevy_platform::collections::HashSet;

pub fn clear_chunks(
    mut cmd: Commands,
    mut mr_clear_layers: MessageReader<ClearScatterLayer>,
    q_children: Query<&Children>,
    q_instances: Query<(Entity, &ScatteredInstance)>,
    q_chunks: Query<Entity, With<Chunk>>,
) {
    if mr_clear_layers.is_empty() {
        return;
    }

    let layers: HashSet<Entity> = mr_clear_layers.read().fold(HashSet::new(), |mut acc, e| {
        acc.insert(**e);
        acc
    });

    for child in q_chunks
        .iter()
        .filter_map(|chunk| q_children.get(chunk).ok())
        .flatten()
        .filter_map(|child| {
            q_instances
                .get(*child)
                .ok()
                .and_then(|(_, instance)| layers.contains(&**instance).then_some(child))
        })
    {
        cmd.entity(*child).despawn();
    }
}
