use crate::asset::backend::systems::{insert_parts, insert_requests};
use crate::asset::systems::process_requests;
use crate::prelude::*;
use bevy_app::{App, Plugin, PostUpdate, Update};
use bevy_ecs::prelude::*;
use bevy_pbr::prelude::*;
use bevy_state::prelude::*;
use std::marker::PhantomData;

pub struct ScatterAssetRequestPlugin<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    _marker: PhantomData<T>,
}

impl<T> ScatterAssetRequestPlugin<T>
where
    T: ScatterMaterial,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for ScatterAssetRequestPlugin<T>
where
    T: ScatterMaterial,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Plugin for ScatterAssetRequestPlugin<T>
where
    T: ScatterMaterial,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (insert_parts::<T>, insert_requests::<T>).run_if(in_state(ScatterState::Collecting)),
        )
        .add_systems(
            Update,
            process_requests::<T>.run_if(in_state(ScatterState::Collecting)),
        );
    }
}
