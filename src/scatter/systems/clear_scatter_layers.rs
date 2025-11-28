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
    for child in mr_clear_layers
        .read()
        .filter_map(|trigger| {
            #[cfg(feature = "trace")]
            debug!("ClearScatterLayer triggered for layer {:?}", trigger);

            q_children.get(**trigger).ok()
        })
        .flatten()
        .filter_map(|x| q_instances.get(*x).ok())
        .map(|(child, _)| child)
    {
        cmd.entity(child).despawn();
    }
}
