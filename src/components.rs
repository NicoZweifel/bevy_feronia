use bevy::prelude::*;

#[derive(Component)]
pub struct WindAffectedReady;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WindAffected;
