use crate::prelude::Chunk;
use crate::scatter::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Commands;
use bevy_platform::collections::HashSet;

#[cfg(feature = "trace")]
use tracing::debug;

pub fn clear_scatter_layers(
    mut cmd: Commands,
    mut mr_clear_layers: MessageReader<ClearScatterLayer>,
    q_children: Query<&Children>,
    q_instances: Query<(Entity, &ScatteredInstance)>,
) {
    let mut layers: HashSet<Entity> = HashSet::new();

    for child in mr_clear_layers
        .read()
        .filter_map(|trigger| {
            #[cfg(feature = "trace")]
            debug!("ClearScatterLayer triggered for layer {:?}", trigger);

            layers.insert(**trigger);
            q_children.get(**trigger).ok()
        })
        .flatten()
        .filter_map(|x| q_instances.get(*x).ok())
        .map(|(child, _)| child)
    {
        cmd.entity(child).despawn();
    }
}

pub fn clear_chunks(
    mut cmd: Commands,
    mut mr_clear_layers: MessageReader<ClearScatterLayer>,
    q_children: Query<&Children>,
    q_instances: Query<(Entity, &ScatteredInstance)>,
    q_chunks: Query<Entity, With<Chunk>>,
) {
    let layers: HashSet<Entity> = mr_clear_layers.read().fold(HashSet::new(), |mut acc, e| {
        acc.insert(**e);
        acc
    });

    if layers.is_empty() {
        return;
    }

    for child in q_chunks
        .iter()
        .filter_map(|chunk| q_children.get(chunk).ok())
        .flatten()
        .filter_map(|child| {
            q_instances
                .get(*child)
                .ok()
                .and_then(|(_, instance)| layers.contains(&**instance).then(|| child))
        })
    {
        cmd.entity(*child).despawn();
    }
}
