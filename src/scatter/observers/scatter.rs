use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    trigger: On<Scatter<TIn, TOut>>,
    mut cmd: Commands,
) {
    let layer_entity = trigger.entity;

    debug!("Scattering Layer: {layer_entity}");

    cmd.entity(layer_entity)
        .insert(ScatterRequest::<TIn, TOut>::new(
            layer_entity,
            layer_entity,
            None,
        ));
}
