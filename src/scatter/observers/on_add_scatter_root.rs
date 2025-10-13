use crate::prelude::*;
use crate::scatter::observers::{hierarchical_scatter, scatter_root};
use bevy::prelude::*;

pub fn on_add_scatter_root<TOut, TIn>(trigger: On<Add, ScatterRoot>, mut cmd: Commands)
where
    TOut: ScatterMaterial<TOut, TIn> + Asset + Clone,
    TIn: Material,
{
    let root = trigger.entity;

    debug!("Added ScatterRoot {root}.");

    cmd.entity(root)
        .observe(scatter_root::<TOut, TIn>)
        .observe(hierarchical_scatter::<TOut, TIn>);
}
