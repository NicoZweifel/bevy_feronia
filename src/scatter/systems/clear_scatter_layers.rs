use crate::prelude::Chunk;
use crate::scatter::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub fn clear_scatter_layers(
    mut cmd: Commands,
    mut mr_clear_layers: MessageReader<ClearScatterLayer>,
    q_children: Query<&Children>,
    q_instances: Query<(Entity, &ScatteredInstance)>,
    q_chunks: Query<Entity, With<Chunk>>,
) {
    let mut layers: HashSet<Entity> = HashSet::new();

    for trigger in mr_clear_layers.read() {
        let layer_entity = trigger.0;

        debug!("ClearScatterLayer triggered for layer {:?}", layer_entity);

        layers.insert(layer_entity);

        let Ok(children) = q_children.get(layer_entity) else {
            continue;
        };

        for (child_entity, _) in children.iter().filter_map(|c| q_instances.get(c).ok()) {
            cmd.entity(child_entity).despawn();
        }
    }

    if layers.is_empty() {
        return;
    }

    for chunk in q_chunks.iter() {
        let Ok(children) = q_children.get(chunk) else {
            continue;
        };

        for child in children.iter().filter(|c| {
            q_instances
                .get(*c)
                .map(|(_, x)| layers.contains(&**x))
                .unwrap_or(false)
        }) {
            cmd.entity(child).despawn();
        }
    }
}
