use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct WindAffectedReady;

#[derive(Component, Deref)]
pub struct WindAffectedRegistered<M>(pub Handle<M>)
where
    M: Asset + Clone;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WindAffected;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WindConfig {
    // NOTE: If set, this material type will be manually controlled and not updated automatically with the global wind resource.
    pub wind_override: Option<Wind>,
}
