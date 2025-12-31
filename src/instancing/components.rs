use bevy_color::Color;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

/// Controls the exponent in the Blinn-Phong specular highlight model.
///
/// Determines the "shininess" of the surface.
/// * **Higher values** (e.g., 32.0, 64.0) result in smaller, tighter, sharper highlights (wet/glossy look).
/// * **Lower values** (e.g., 4.0, 8.0) result in broader, duller highlights (rough surface).
///
/// Maps to `material_uniforms.specular_power` in the shader.
///
/// Defaults to `32.0`.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct SpecularPower(pub f32);

impl Default for SpecularPower {
    fn default() -> Self {
        Self(32.0)
    }
}

/// Enabled a fake Ambient Occlusion effect.
///
/// Enables `#ifdef AMBIENT_OCCLUSION` in shaders.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct AmbientOcclusion;

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
/// Only supported with [`InstancedWindAffectedMaterial`].
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
/// Corresponds to `material_uniforms.translucency` in the shader.
/// Defaults to `0.6`.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
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
/// Enables `#ifdef DIRECTIONAL_LIGHTS` in shaders.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(Translucency, SpecularPower, SpecularStrength)]
pub struct DirectionalLights;

/// Marker component to enable point lights.
///
/// Enables `#ifdef POINT_LIGHTS` in shaders.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(Translucency, SpecularPower, SpecularStrength)]
pub struct PointLights;

/// Controls the normal curving effect (simulates curved blades).
///
/// This is a multiplier that determines how strongly the blade curves from its
/// center (0.0) to its edges (1.0), using the **`uv.x`** coordinate of the mesh.
/// The final curve angle is hard-capped at **1.4 radians (~80°)**.
///
/// ### UV Requirements
/// **Important:** This feature relies on specific UV mapping:
///
/// **Horizontal (`uv.x`):** Must range from **0.0 to 1.0**.
///     * `0.0`: Left edge
///     * `0.5`: Center spine
///     * `1.0`: Right edge
///
/// If `uv.x` isn't mapped to this range, the curve math (`x * 2.0 - 1.0`) will produce
/// incorrect normal offsets and visual artifacts.
///
/// ### Behavior
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
/// ### Notes
///
/// Corresponds to `material_uniforms.curve_factor` in shaders.
///
/// Defaults to `0.3`.
///
/// Currently only supported with [`InstancedWindAffectedMaterial`].
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
/// A higher value will apply a stronger Bézier curve and will affect the instances more uniformly,
/// while a lower value will affect them more randomly and apply less curve.
///
/// Corresponds to `material_uniforms.static_bend_strength` in shaders.
///
/// Defaults to `0.5`.
///
/// Currently only supported with [`InstancedWindAffectedMaterial`] but should be easy to add.
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct StaticBendStrength(pub f32);

impl Default for StaticBendStrength {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Sets the material base color.
#[derive(Component, Clone, Copy, Debug, Reflect, Default, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct InstanceColor(pub Color);

/// Sets a material color gradient.
///
/// ### UV Requirements
///
/// **Important:** This feature relies on specific UV mapping:
///
/// **Vertical (`uv.y`):** Must range from **0.0 to 1.0**.
///    - `0.0`: **Tip / Top** of the mesh.
///    - `1.0`: **Root / Base** of the mesh.
///
/// #### Troubleshooting:
///   - If you have black spots or other artifacts on your mesh, `uv.y` might not be ranging from `0.0` to `1.0`.
///
///   - If your colors appear upside down (Top Color at the bottom), your mesh likely uses
///     UVs where `0.0` is at the bottom. You should flip the UVs vertically (so the root is at `1.0`).
///
/// ### Notes
///
/// Corresponds to the following uniforms in shaders.
///   - `material.bottom_color`
///   - `material.top_color`
///   - `material.tint_factor`
///   - `material.gradient_start`
///   - `material.gradient_end`
///
/// Currently only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
pub struct InstanceColorGradient {
    /// The top color if the gradient.
    pub top: Color,
    /// The bottom color of the gradient.
    pub bottom: Color,
    /// The tint factor of the gradient.
    /// - 0.0 = no tint
    /// - 1.0 = full tint
    pub tint: f32,
    /// The height (0.0 to 1.0) where the bottom color stops being solid
    /// and the gradient begins transitioning to be top-colored.
    pub start: f32,
    /// The height (0.0 to 1.0) where the gradient finishes, becoming fully top-colored.
    pub end: f32,
}

impl InstanceColorGradient {
    pub fn new(top: Color, bottom: Color) -> Self {
        Self {
            top,
            bottom,
            tint: 1.0,
            start: 0.0,
            end: 1.0,
        }
    }
}
