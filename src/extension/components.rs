use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

/// Disables displacement on shadows.
///
/// Enables `#ifdef STATIC_SHADOWS` in shaders.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
///
/// [`InstancedWindAffectedMaterial`] has no support for shadows currently,
/// because it is not using the standard PBR lighting.
///
/// TODO: https://github.com/NicoZweifel/bevy_feronia/issues/49
#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component, Debug, Clone)]
pub struct StaticShadow;

/// Marker to make base [`StandardMaterial`] unlit.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct Unlit;
