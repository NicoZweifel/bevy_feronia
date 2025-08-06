use crate::core::ScatterAsset;
use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct WindAffectedReady;

#[derive(Component)]
pub struct WindAffectedRegistered<M>(pub ScatterAsset<M>)
where
    M: Asset + Clone;

impl<M> WindAffectedRegistered<M>
where
    M: Asset + Clone,
{
    pub fn get(&self) -> ScatterAsset<M> {
        self.0.clone()
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WindAffected;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WindConfig {
    // NOTE: If set, this material type will be manually controlled and not updated automatically with the global wind resource.
    pub wind_override: Option<Wind>,
}
