use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter<TOut: ScatterMaterial<TIn>, TIn: Material>(
    trigger: On<Scatter<TOut, TIn>>,
    mut cmd: Commands,
) {
    let layer_entity = trigger.entity;

    debug!("Scattering Layer: {layer_entity}");

    cmd.entity(layer_entity)
        .insert(ScatterRequest::<TOut, TIn>::new(
            layer_entity,
            layer_entity,
            None,
        ));
}
