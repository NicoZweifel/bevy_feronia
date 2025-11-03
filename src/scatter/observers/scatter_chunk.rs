use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter_chunk<TOut, TIn>(
    trigger: On<ScatterChunk<TOut, TIn>>,
    mut cmd: Commands,
    q_chunk: Query<Entity, (With<Chunk>, Without<Merging>)>,
) where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    let chunk_entity = trigger.entity;

    if q_chunk.get(chunk_entity).is_err() {
        return;
    }

    cmd.entity(chunk_entity)
        .insert(ScatterRequest::<TOut, TIn>::new(
            chunk_entity,
            trigger.scatter_layer,
            Some(chunk_entity),
        ));
}
