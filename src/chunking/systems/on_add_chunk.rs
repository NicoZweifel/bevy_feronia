use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn on_add_chunk(trigger: On<Add, Chunk>, mut cmd: Commands) {
    let chunk = trigger.target();

    debug!("Chunk added: {chunk}.");

    cmd.entity(chunk).insert(ChunkInitialize);
}
