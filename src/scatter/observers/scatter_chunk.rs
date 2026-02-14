use crate::prelude::*;
use bevy_ecs::prelude::*;

pub fn scatter_chunk<T>(
    trigger: On<ScatterChunk<T>>,
    mut cmd: Commands,
    q_chunk: Query<Entity, (With<Chunk>, Without<Merging>)>,
) where
    T: ScatterMaterial,
{
    let chunk_entity = trigger.entity;
    if q_chunk.get(chunk_entity).is_err() {
        return;
    }

    cmd.entity(chunk_entity).insert(ScatterRequest::<T>::new(
        chunk_entity,
        trigger.scatter_layer,
        Some(chunk_entity),
    ));
}
