use crate::prelude::*;
use bevy_ecs::prelude::*;

#[cfg(feature = "tracing")]
use tracing::debug;

pub fn scatter<T>(trigger: On<Scatter<T>>, mut cmd: Commands)
where
    T: ScatterMaterial,
{
    let layer_entity = trigger.entity;

    #[cfg(feature = "tracing")]
    debug!("Scattering Layer: {layer_entity}");

    cmd.entity(layer_entity)
        .insert(ScatterRequest::<T>::new(layer_entity, layer_entity, None));
}
