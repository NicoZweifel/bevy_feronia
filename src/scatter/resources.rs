use crate::prelude::ScatterMaterial;
use crate::prelude::events::*;

use bevy_derive::*;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use std::collections::VecDeque;

/// A global resource to seed the entire world generation process.
/// Changing this value will generate a completely different world.
#[derive(Resource, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Resource, Debug, Clone)]
pub struct WorldSeed(pub u64);

impl WorldSeed {
    pub fn get(&self) -> u64 {
        **self
    }
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self(123456789)
    }
}

#[derive(Resource, Default, DerefMut, Deref, Debug, Reflect)]
#[reflect(Resource, Debug)]
pub struct SpawnScatterAssetsEventQueue<T: ScatterMaterial>(pub VecDeque<SpawnScatterAssets<T>>);
