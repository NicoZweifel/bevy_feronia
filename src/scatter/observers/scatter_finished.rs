use crate::prelude::*;
use bevy_ecs::prelude::*;

pub fn scatter_finished<T>(trigger: On<Remove, HierarchicalScatterState<T>>, mut cmd: Commands)
where
    T: ScatterMaterial,
{
    println!("Scatter finished");
    cmd.trigger(ScatterFinished::<T>::from(trigger.entity));
}
