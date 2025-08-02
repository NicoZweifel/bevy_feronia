use crate::prelude::*;
use crate::scatter::observers::scatter_chunk;
use bevy::prelude::*;

pub fn on_add_chunk(trigger: On<Add, Chunk>, mut cmd: Commands) {
    debug!("Chunk added: {}", trigger.target());

    cmd.entity(trigger.target()).observe(scatter_chunk);
}
