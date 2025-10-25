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

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct StrengthMultiplier(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct MicroStrengthMultiplier(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveStrength(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveSpeed(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveFrequency(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BopStrength(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BopSpeed(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct TwistStrength(pub f32);

#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BendExponent(pub f32);

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct LowQuality;
