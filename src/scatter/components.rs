use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterItem;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ScatterRootReady;

#[derive(Component, Reflect, Debug, Clone, Deref)]
#[reflect(Component)]
pub struct ScatterItemAsset<T>(pub Handle<ScatterAsset<T>>)
where
    T: Asset + Clone;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterLayer)]
pub struct ScatterItemOf(pub Entity);

#[derive(Component, Reflect, Default)]
#[require(Transform, Visibility, GlobalTransform)]
#[relationship_target(relationship = ScatterItemOf)]
#[reflect(Component)]
pub struct ScatterLayer(Vec<Entity>);

/// A marker component to signify that a `ScatterLayer` has already had its
/// sources discovered and its `ScatterItem's generated.
#[derive(Component)]
pub struct ScatterLayerProcessed;

#[derive(Component, Debug, Clone, Reflect, Deref)]
#[reflect(Component)]
#[relationship(relationship_target = ScatterRoot)]
pub struct ScatterLayerOf(pub Entity);

#[derive(Component, Debug, Clone, Reflect, Deref, Default)]
#[reflect(Component)]
#[require(Transform, Visibility, GlobalTransform, LodConfig)]
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
