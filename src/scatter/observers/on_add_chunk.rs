use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn on_add_chunk(
    trigger: On<Add, Chunk>,
    mut cmd: Commands,
    q_chunk: Query<&ChunkOf, With<Chunk>>,
    q_root: Query<&ScatterRoot>,
) {
    info!("Chunk added: {}", trigger.target());

    cmd.entity(trigger.target()).observe(scatter_chunk);

    let Ok(root_chunk) = q_chunk.get(trigger.target()) else {
        warn!("Couldn't get Chunk: {}", trigger.target());
        return;
    };

    let Ok(root) = q_root.get(**root_chunk) else {
        warn!("Couldn't get ScatterRoot: {}", trigger.target());
        return;
    };

    for scatter_layer in root.iter() {
        cmd.trigger_targets(ScatterChunk { scatter_layer }, [trigger.target()])
    }
}
