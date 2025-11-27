use crate::prelude::LodDensity;
use bevy_color::{Color, LinearRgba};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryItem;
use bevy_math::{Vec3, Vec4};
use bevy_reflect::Reflect;
use bevy_render::render_resource::Buffer;
use bevy_render::{extract_component::ExtractComponent, render_resource::BindGroup};
use bevy_utils::default;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// Controls the exponent in the Blinn-Phong specular highlight model.
///
/// Determines the "shininess" of the surface.
/// * **Higher values** (e.g., 32.0, 64.0) result in smaller, tighter, sharper highlights (wet/glossy look).
/// * **Lower values** (e.g., 4.0, 8.0) result in broader, duller highlights (rough surface).
///
/// Maps to `specular_power` in the shader.
///
/// Defaults to `32.0`.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct SpecularPower(pub f32);

impl Default for SpecularPower {
    fn default() -> Self {
        Self(32.0)
    }
}

/// Controls the intensity of the specular highlight.
///
/// Acts as a multiplier for the specular light contribution.
/// * `0.0`: No specular highlights (purely diffuse).
/// * `1.0`: Full brightness specular highlights.
///
/// Maps to `specular_strength` in the shader.
///
/// Defaults to `0.6`.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct SpecularStrength(pub f32);

impl Default for SpecularStrength {
    fn default() -> Self {
        Self(0.6)
    }
}

/// Controls the amount of light that simulates passing through the object.
///
/// Used to simulate thin geometry like grass blades or leaves being lit from behind.
/// It scales the lighting contribution when the light direction is opposite the surface normal.
///
/// * `0.0`: Opaque (no light passes through).
/// * Higher values increase the brightness of backlit surfaces.
///
/// Maps to `translucency` in the shader.
/// Defaults to `0.6`.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct Translucency(pub f32);

impl Default for Translucency {
    fn default() -> Self {
        Self(0.6)
    }
}

/// Marker component to enable directional lights.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
///
/// Enables `#ifdef DIRECTIONAL_LIGHTS` in shaders.
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(Translucency, SpecularPower, SpecularStrength)]
pub struct DirectionalLights;

/// Marker to make base [`StandardMaterial`] unlit.
///
/// Only supported with [`ExtendedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct Unlit;

/// Marker component to enable point lights.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
///
/// Enables `#ifdef POINT_LIGHTS` in shaders.
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(Translucency, SpecularPower, SpecularStrength)]
pub struct PointLights;

/// Controls the edge correction effect (makes vegetation look fuller).
///
/// Corresponds to `wind.edge_correction_factor` in shaders.
///
/// Defaults to `0.02`.
///
/// Not supported in combination with [`EnableBillboarding`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct EdgeCorrectionFactor(pub f32);

impl Default for EdgeCorrectionFactor {
    fn default() -> Self {
        Self(0.02)
    }
}

/// Controls the normal curving effect (simulates curved blades).
///
/// This is a multiplier that determines how strongly the blade curves from its
/// center (0.0) to its edges (1.0), using the **`uv.x`** coordinate of the mesh.
/// The final curve angle is hard-capped at **1.4 radians (~80°)**.
///
/// The behavior changes significantly depending on the value:
///
/// - **`CurveFactor < 1.4`**: Creates a gentle, shallow curve. The blade will
///   never reach the 80° maximum, resulting in a softer look.
///
/// - **`CurveFactor = 1.4`**: Creates a full, linear curve. The blade
///   bends smoothly from 0° at the center to the 80° maximum exactly at the edge.
///
/// - **`CurveFactor > 1.4`**: The blade hits the 80° cap *before* reaching the edge.
///   This creates a sharp "crease" or "spine" down the center, with the rest
///   of the blade remaining flat at the maximum angle.
///
/// Corresponds to `wind.curve_factor` in shaders.
///
/// Defaults to `0.3`.
///
/// Currently only supported in [`InstancedWindAffectedMaterial`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct CurveFactor(pub f32);

impl Default for CurveFactor {
    fn default() -> Self {
        Self(0.3)
    }
}

/// Controls a persistent, non-wind bend.
///
/// Corresponds to `instance_uniforms.static_bend_strength` in shaders.
///
/// A higher value will apply a stronger Bézier curve and will affect the instances more uniformly,
/// while a lower value will affect them more randomly and apply less curve.
///
/// Defaults to `0.5`.
///
/// Currently only supported in [`InstancedWindAffectedMaterial`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct StaticBendStrength(pub f32);

impl Default for StaticBendStrength {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Marker component to opt in to GPU-driven culling/preparation.
#[derive(Component, Clone, Copy, Default, ExtractComponent)]
pub struct GpuCull;

/// Used for the Compute Shader culling logic.
#[derive(Component, Clone, Copy, Debug, Reflect, Deref, DerefMut, ExtractComponent)]
#[reflect(Component)]
pub struct CullLodDensity(pub f32);

impl Default for CullLodDensity {
    fn default() -> Self {
        Self(1.0)
    }
}

impl From<LodDensity> for CullLodDensity {
    fn from(lod_density: LodDensity) -> Self {
        Self(*lod_density)
    }
}

/// Sets a material color tint.
///
/// Corresponds to `instance_uniforms.color` in shaders.
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct InstanceColor {
    pub top: Color,
    pub bottom: Color,
}

impl InstanceColor {
    pub fn new(top: Color, bottom: Color) -> Self {
        Self { top, bottom }
    }
}

#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub(crate) struct InstancePipelineKey(pub u64);

impl ExtractComponent for InstancePipelineKey {
    type QueryData = &'static InstancePipelineKey;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(*item)
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub index: u32,
    pub _padding: [u32; 3],
}

#[derive(Component, Clone)]
pub struct InstanceMaterialData {
    pub instances: Arc<Vec<InstanceData>>,
    pub top_color: LinearRgba,
    pub bottom_color: LinearRgba,
    pub visibility_range: Vec4,
    pub static_bend_strength: f32,
    pub curve_factor: f32,
    pub translucency: f32,
    pub specular_strength: f32,
    pub specular_power: f32,
}

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

#[derive(Component)]
pub struct GpuDrawIndexedIndirect {
    pub buffer: Buffer,
    pub offset: u64,
}

#[derive(Component)]
pub struct InstanceLodBuffer {
    pub buffer: Buffer,
}

#[derive(Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct InstanceUniforms {
    pub top_color: LinearRgba,

    pub bottom_color: LinearRgba,

    pub visibility_range: Vec4,

    pub static_bend_strength: f32,
    pub curve_factor: f32,
    pub translucency: f32,
    pub specular_strength: f32,

    pub specular_power: f32,
    pub _padding: [f32; 3],
}

impl From<&InstanceMaterialData> for InstanceUniforms {
    fn from(value: &InstanceMaterialData) -> Self {
        InstanceUniforms {
            top_color: value.top_color,
            bottom_color: value.bottom_color,
            visibility_range: value.visibility_range,
            static_bend_strength: value.static_bend_strength,
            curve_factor: value.curve_factor,
            translucency: value.translucency,
            specular_power: value.specular_power,
            specular_strength: value.specular_strength,
            ..default()
        }
    }
}

#[derive(Component)]
pub struct InstanceUniformBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub struct InstancedComputeSourceBuffer {
    pub buffer: Buffer,
    pub count: u32,
}

#[derive(Component)]
pub struct InstancedComputeBindGroup(pub BindGroup);
