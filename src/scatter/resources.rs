use bevy_derive::*;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

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
