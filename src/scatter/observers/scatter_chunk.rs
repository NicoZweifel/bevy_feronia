use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

pub fn scatter_chunk<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut trigger: On<ScatterChunk<TIn, TOut>>,
    mut cmd: Commands,
    q_chunk: Query<Entity, (With<Chunk>, Without<Merging>)>,
) {
    trigger.propagate(false);

    let chunk_entity = trigger.target();

    if q_chunk.get(chunk_entity).is_err() {
        return;
    }

    cmd.entity(chunk_entity)
        .insert(ScatterRequest::<TIn, TOut>::new(
            chunk_entity,
            trigger.scatter_layer,
            Some(chunk_entity),
        ));
}
