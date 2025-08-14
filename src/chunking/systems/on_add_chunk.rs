use crate::prelude::*;
use bevy::prelude::*;

pub fn on_add_chunk(trigger: On<Add, Chunk>, mut cmd: Commands) {
    let chunk = trigger.target();

    debug!("Chunk added: {chunk}.");

    cmd.entity(chunk).insert(ChunkInitialize);
}


pub fn chunk_init(
    mut cmd: Commands,
    q_chunks: Query<Entity, (With<Chunk>,With<ChunkInitialize>)>,
) {
    for chunk in q_chunks.iter() {
        debug!("Chunk initialized: {chunk}.");

        cmd.entity(chunk).remove::<ChunkInitialize>();
    }
}
