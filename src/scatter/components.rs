use bevy::asset::Handle;
use bevy::image::Image;
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[require(Transform, Visibility, GlobalTransform)]
#[reflect(Component)]
pub struct ScatterLayer;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, GlobalTransform)]
#[relationship_target(relationship = ScatterLayerOf)]
pub struct ScatterRoot(Vec<Entity>);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct ScatterLayerEnabled(pub bool);

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct DistributionDensity(pub f32);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DistributionPattern {
    pub density_map: Handle<Image>,
    pub scale: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InstanceRotationYaw {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InstanceScale {
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct InstanceJitter(pub f32);
