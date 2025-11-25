use crate::prelude::*;
use crate::scatter::observers::*;
use bevy_ecs::prelude::*;

#[cfg(feature = "tracing")]
use tracing::debug;

pub fn on_add_scatter_root<T>(trigger: On<Add, ScatterRoot>, mut cmd: Commands)
where
    T: ScatterMaterial,
{
    let root = trigger.entity;

    #[cfg(feature = "tracing")]
    debug!("Added ScatterRoot {root}.");

    cmd.entity(root)
        .observe(scatter_root::<T>)
        .observe(hierarchical_scatter::<T>)
        .observe(scatter_observer::<T>);
}
