use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_render::extract_component::ExtractComponent;
use derive_more::From;

/// Marker component identifying the entity representing the center of the chunking and lod systems.
///
/// This should be added to the camera or the player controller.
#[derive(Component, Reflect, Clone, ExtractComponent)]
#[reflect(Component)]
pub struct Center;

/// Component specifying the LOD for a [`ScatterItem`].
#[derive(
    Component,
    Deref,
    DerefMut,
    Clone,
    Copy,
    Debug,
    Default,
    Reflect,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    From,
)]
#[reflect(Component, Clone, Debug, PartialEq, Hash)]
pub struct LevelOfDetail(pub u32);

impl LevelOfDetail {
    pub fn is_highest_detail(&self) -> bool {
        self.0 == 0
    }
}

/// Marker component for debug visualization.
///
/// Makes shaders return `debug_color` in the fragment shader.
///
/// Enables `#ifdef MATERIAL_DEBUG` in shaders.
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct EnableDebug;

/// Marker component to make instances always face the camera.
///
/// Enables `#ifdef BILLBOARDING` in shaders.
///
/// Not supported in combination with [`EdgeCorrectionFactor`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct EnableBillboarding;

/// Marker component to force simple, undisplaced world-space normals.
///
/// Will have incorrect lighting on displaced vertices,
/// as the normals will not match the displaced vertex positions.
///
/// **Note:** If neither [`FastNormals`] nor [`AnalyticalNormals`] is present,
/// the shader defaults to the numerical path, which is the most accurate, but most expensive path,
/// as it runs the full displacement logic on the neighbors to find the surface direction,
/// which should only be used for complex foliage like non-billboarded bushes, trees.
///
/// **Note:** For correct fallback behavior (if the mesh lacks tangents or normals),
/// the mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
/// and its "face" pointing along Z-Up (`+Z`).
///
/// Typically used for performance reasons and/or on static or barely wind-affected objects.
///
/// Enables `#ifdef FAST_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct FastNormals;

/// Marker component to enable approximated, mathematically derived normals.
///
/// Should be faster than numerical sampling but less accurate,
/// as it only accounts for `static_bend`, `twist`,
/// and `macro_wind`, ignoring high-frequency displacements.
///
/// **Note:** If neither [`FastNormals`] nor [`AnalyticalNormals`] is present,
/// the shader defaults to the numerical path, which is the most accurate, but most expensive path,
/// as it runs the full displacement logic on the neighbors to find the surface direction,
/// which should only be used for complex foliage like non-billboarded bushes, trees.
///
/// **Note:** For correct fallback behavior (if the mesh lacks tangents or normals),
/// the mesh should ideally be modeled with its "growth" axis along Y-Up (`+Y`)
/// and its "face" pointing along Z-Up (`+Z`).
///
/// Typically used for billboarded foliage or flat meshes like grass.
///
/// Enables `#ifdef ANALYTICAL_NORMALS` in shaders.
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct AnalyticalNormals;

/// Adds Edge Correction.
///
/// See [`EdgeCorrectionFactor`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(EdgeCorrectionFactor)]
pub struct EdgeCorrection;

/// Controls the edge correction effect (makes vegetation look fuller).
///
/// Corresponds to `wind.edge_correction_factor` in shaders.
///
/// Defaults to `0.02`.
///
/// Enables `#ifdef EDGE_CORRECTION` in shaders if larger than 0.
///
/// Not supported in combination with [`EnableBillboarding`].
///
/// TODO requires previous camera/view and normals so currently broken with temporal fx
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct EdgeCorrectionFactor(pub f32);

impl Default for EdgeCorrectionFactor {
    fn default() -> Self {
        Self(0.02)
    }
}

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
/// [`InstancedWindAffectedMaterial`] has support for subsurface scattering currently,
/// if it is using the standard PBR lighting, which this effect requires (emissive material).
#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component, Debug, Clone)]
#[require(
    SubsurfaceScatteringIntensity,
    SubsurfaceScatteringScale,
    LightIntensity
)]
pub struct SubsurfaceScattering;

/// Controls the overall intensity of the subsurface scattering (SSS) effect.
///
/// This acts as a master multiplier for the entire SSS calculation,
/// scaling both the back-scatter and front-scatter components.
/// Higher values make the material more emissive, i.e., it starts to `glow`.
///
/// Corresponds to `sss_strength` in the shader uniforms.
///
/// Defaults to `0.5`.
///
/// **Note:** This component only has an effect if the
/// [`SubsurfaceScattering`] marker component is also present.
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Debug, Clone)]
pub struct SubsurfaceScatteringIntensity(pub f32);

impl Default for SubsurfaceScatteringIntensity {
    fn default() -> Self {
        Self(0.5)
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
/// Defaults to `1.0`.
///
/// **Note:** This component only has an effect if the
/// [`SubsurfaceScattering`] marker component is also present.
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct SubsurfaceScatteringScale(pub f32);

impl Default for SubsurfaceScatteringScale {
    fn default() -> Self {
        Self(1.)
    }
}

/// Controls the light intensity scale.
///
/// Used for Blinn-Phing shading/lighting and [`StandardPBR`] + [`SubsurfaceScattering`],
/// but should be set to a low value when used with Blinn-Phong, e.g., `0.0001`.
///
/// Defaults to `1.0`.
///
/// **NOTE:** this might be removed in the future.
///
/// TODO
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct LightIntensity(pub f32);

impl Default for LightIntensity {
    fn default() -> Self {
        Self(1.0)
    }
}
