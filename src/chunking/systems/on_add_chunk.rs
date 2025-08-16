use crate::prelude::*;
use bevy::prelude::*;
use rand::SeedableRng;
use std::hash::Hasher;
use std::hash::{DefaultHasher, Hash};

pub fn on_add_chunk(trigger: On<Add, Chunk>, mut cmd: Commands, q_chunk: Query<&ChunkCoord>) {
    let chunk = trigger.target();

    debug!("Chunk added: {chunk}.");

    let Ok(chunk_coord) = q_chunk.get(chunk) else {
        warn!("Chunk {chunk} has no chunk coord. Can't initialize Chunk without Coordinates.");
        return;
    };

    cmd.entity(chunk).insert((ChunkInitialize));
}

pub fn chunk_init(
    mut cmd: Commands,
    q_chunks: Query<Entity, (With<Chunk>, With<ChunkInitialize>)>,
) {
    for chunk in q_chunks.iter() {
        debug!("Chunk initialized: {chunk}.");

        cmd.entity(chunk).remove::<ChunkInitialize>();
    }
}
