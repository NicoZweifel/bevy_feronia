use bevy_ecs::prelude::*;
use bevy_derive::{Deref, DerefMut};
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

#[derive(Bundle, Default, Debug, Clone)]
pub struct SssBundle {
    sss: SubsurfaceScattering,
    intensity: SubsurfaceScatteringIntensity,
    scale: SubsurfaceScatteringScale,
}

impl SssBundle {
    pub fn new(intensity: f32, scale: f32) -> Self {
        Self {
            intensity: SubsurfaceScatteringIntensity(intensity),
            scale: SubsurfaceScatteringScale(scale),
            ..Default::default()
        }
    }
}

/// Marker component to enable simulated subsurface scattering.
///
/// Enables `#ifdef SUBSURFACE_SCATTERING` in shaders.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
///
/// [`InstancedWindAffectedMaterial`] has no support for subsurface scattering currently,
/// because it is not using the standard PBR lighting, which this effect requires (emissive material).
#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component, Debug, Clone)]
#[require(SubsurfaceScatteringIntensity, SubsurfaceScatteringScale)]
pub struct SubsurfaceScattering;

/// Controls the overall intensity of the subsurface scattering (SSS) effect.
///
/// This acts as a master multiplier for the entire SSS calculation,
/// scaling both the back-scatter and front-scatter components.
/// Higher values make the material appear more translucent or "waxy".
///
/// Corresponds to `sss_strength` in the shader uniforms.
///
/// Defaults to `2.0`.
///
/// **Note:** This component only has an effect if the
/// [`SubsurfaceScattering`] marker component is also present.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Debug, Clone)]
pub struct SubsurfaceScatteringIntensity(pub f32);

impl Default for SubsurfaceScatteringIntensity {
    fn default() -> Self {
        Self(2.0)
    }
}

/// Controls the strength of the back-scattering (rim-lighting) SSS effect.
///
/// This value specifically scales the light that scatters *through* the
/// object from behind (relative to the camera). Higher values create
/// a brighter, more pronounced "glow" on the edges of the object
/// when it is backlit.
///
/// Corresponds to `sss_scale` in the shader uniforms.
///
/// Defaults to `1.5`.
///
/// **Note:** This component only has an effect if the
/// [`SubsurfaceScattering`] marker component is also present.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct SubsurfaceScatteringScale(pub f32);

impl Default for SubsurfaceScatteringScale {
    fn default() -> Self {
        Self(1.5)
    }
}
