use crate::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

/// Marker component to enable wind effects for an entity.
///
/// Enables `#ifdef WIND_AFFECTED` in shaders, applying all wind-related vertex displacements.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Clone)]
pub struct WindAffected;

/// Component for configuring wind behavior on a per-entity basis.
///
/// Might be deprecated/removed in the future, once the API is more mature.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WindConfig {
    /// If `Some`, this [`Wind`] struct overrides the global wind settings for this entity.
    /// This allows for manually controlled or localized wind effects.
    ///
    /// If set, this material type will be manually controlled and not updated automatically with the global wind resource.
    // TODO fix this und update options/material settings properly
    pub wind_override: Option<Wind>,
}

/// Component to multiply the `strength` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.strength` in shaders, controlling the main macro wind displacement strength.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Strength(pub f32);

/// Component to multiply the `micro_strength` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.micro_strength` in shaders, controlling the high-frequency noise displacement.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct MicroStrength(pub f32);

/// Component to multiply the `s_curve_strength` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.s_curve_strength` in shaders, controlling the 'S' shape wiggle size.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveStrength(pub f32);

/// Component to multiply the `s_curve_speed` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.s_curve_speed` in shaders, controlling the 'S' shape wiggle animation speed.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveSpeed(pub f32);

/// Component to multiply the `s_curve_frequency` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.s_curve_frequency` in shaders, controlling the spatial frequency (tiling)
/// of the 'S' shape wiggles along the Y-axis.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct SCurveFrequency(pub f32);

/// Component to multiply the `bop_strength` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.bop_strength` in shaders, controlling the size of the vertical 'bop' animation.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BopStrength(pub f32);

/// Component to multiply the `bop_speed` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.bop_speed` in shaders, controlling the speed of the vertical 'bop' animation.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BopSpeed(pub f32);

/// Component to multiply the `twist_strength` field of an entity's [`Wind`] settings.
///
/// Corresponds to `wind.twist_strength` in shaders, controlling the twist applied by the macro wind.
///
/// This effect is disabled if `BILLBOARDING` is active.
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct TwistStrength(pub f32);

/// Marker component to disable secondary wind effects for performance.
///
/// Enables `#ifdef WIND_LOW_QUALITY` in shaders, which disables micro-noise, S-curve,
/// and bop calculations.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct LowQuality;
