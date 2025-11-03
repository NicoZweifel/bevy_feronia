use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter_finished<TOut, TIn>(
    trigger: On<Remove, HierarchicalScatterState<TOut, TIn>>,
    mut cmd: Commands,
) where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    println!("Scatter finished");
    cmd.trigger(ScatterFinished::<TOut, TIn>::from(trigger.entity));
}
