use bevy::color::Color;
use bevy::prelude::*;

/// Component specifying the LOD for a [`ScatterItem`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LevelOfDetail(pub u32);

/// Marker component for debug visualization.
///
/// Enables `#ifdef MATERIAL_DEBUG` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableDebug;

/// Marker component to make instances always face the camera.
///
/// Enables `#ifdef WIND_BILLBOARDING` in shaders.
///
/// Not supported in combination with [`EdgeCorrectionFactor`].
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableBillboarding;

/// Marker component to force simple, non-analytical normal calculation.
///
/// Enables `#ifdef FAST_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct FastNormals;

/// Marker component to enable high-quality, mathematically derived normals.
///
/// Enables `#ifdef ANALYTICAL_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct AnalyticalNormals;

/// Marker component to enable point lights.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
///
/// Enables `#ifdef POINT_LIGHTS` in shaders.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct PointLights;

/// Controls the edge correction effect (makes vegetation look fuller).
///
/// Corresponds to `wind.edge_correction_factor` in shaders.
///
/// Not supported in combination with [`EnableBillboarding`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct EdgeCorrectionFactor(pub f32);

impl Default for EdgeCorrectionFactor {
    fn default() -> Self {
        Self(0.05)
    }
}

/// Controls the normal curving effect (simulates curved blades).
///
/// This value represents the maximum curve angle (in radians).
/// A value of `1.4` would result in a maximum curve of ~80 degrees.
///
/// Corresponds to `wind.curve_factor` in shaders.
///
/// Currently only supported in `[InstancedWindAffectedMaterial]`.`
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct CurveFactor(pub f32);

impl Default for CurveFactor {
    fn default() -> Self {
        Self(0.1)
    }
}

/// Controls a persistent, non-wind bend.
///
/// Corresponds to `instance_uniforms.static_bend_strength` in shaders.
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct StaticBendStrength(pub f32);

impl Default for StaticBendStrength {
    fn default() -> Self {
        Self(0.3)
    }
}

/// Marker component to enable simulated subsurface scattering.
///
/// Enables `#ifdef SUBSURFACE_SCATTERING` in shaders.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct SubsurfaceScattering;

/// Sets a material color tint.
///
/// Corresponds to `instance_uniforms.color` in shaders.
#[derive(Component, Clone, Debug, Reflect, Deref, Default, DerefMut)]
#[reflect(Component)]
pub struct InstanceColor(pub Color);
