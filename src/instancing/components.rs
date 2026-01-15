use bevy_color::Color;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_math::Vec2;
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
#[derive(Component, Clone, Debug, Reflect, Default)]
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
/// Defaults to `0.5`.
///
/// Only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component, Clone, Debug)]
pub struct SpecularStrength(pub f32);

impl Default for SpecularStrength {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Controls the amount of light that is simulated passing through the object.
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

/// Adds a normal curving effect (simulates curved blades).
///
/// See [`CurveFactor`].
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(CurveFactor)]
pub struct CurveNormals;

/// Controls the normal curving effect.
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
/// Enables `#ifdef CURVE_NORMALS` in shaders if larger than 0.
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

/// Enables Bézier curve bending.
///
/// See [`StaticBendStrength`], [`StaticBendMinMax`], [`StaticBendDirection`] and [`StaticBendControlPoint`].
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component, Clone, Debug)]
#[require(StaticBendStrength)]
pub struct StaticBend;

/// Controls a persistent, non-wind bend.
///
/// A higher value will apply a stronger Bézier curve and will affect the instances more uniformly,
/// while a lower value will affect them more randomly and apply less curve.
///
/// Corresponds to `material_uniforms.static_bend_strength` in shaders.
///
/// Defaults to `0.5`.
///
/// Enables `#ifdef STATIC_BEND` in shaders if larger than 0.
///
/// Currently only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
#[require(StaticBendDirection, StaticBendControlPoint, StaticBendMinMax)]
pub struct StaticBendStrength(pub f32);

impl Default for StaticBendStrength {
    fn default() -> Self {
        Self(0.5)
    }
}

/// Controls the minimum and maximum values for the static bend.
///
/// Corresponds to `material_uniforms.static_bend_min_max` in shaders.
///
/// Currently only supported with [`InstancedWindAffectedMaterial`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct StaticBendMinMax {
    min: f32,
    max: f32,
}

impl StaticBendMinMax {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

impl Into<Vec2> for StaticBendMinMax {
    fn into(self) -> Vec2 {
        Vec2::new(self.min, self.max)
    }
}

impl From<Vec2> for StaticBendMinMax {
    fn from(vec: Vec2) -> Self {
        Self {
            min: vec.x,
            max: vec.y,
        }
    }
}

impl Default for StaticBendMinMax {
    fn default() -> Self {
        Self { min: 0.3, max: 0.9 }
    }
}

/// Controls the direction of the static bend.
///
/// Corresponds to `material_uniforms.bend_direction` in the shader.
///
/// Defaults to: `(0.309017, -0.951056)`.
#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct StaticBendDirection(pub Vec2);

impl Default for StaticBendDirection {
    fn default() -> Self {
        Self(Vec2::new(0.309017, -0.951056))
    }
}

impl StaticBendDirection {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

/// Controls the Bézier control points.
///
/// * **Stiffness**: Controls how far out the control point is. Lower values make the curve "tighter" at the base. Defaults to `0.33`.
/// * **Height**: Controls the vertical height of the control point relative to the mesh height. Defaults to `0.55`.
///
/// Corresponds to `material_uniforms.bend_control_point` in the shader.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component, Clone, Debug)]
pub struct StaticBendControlPoint {
    pub stiffness: f32,
    pub height: f32,
}

impl Default for StaticBendControlPoint {
    fn default() -> Self {
        Self::new(0.33, 0.55)
    }
}

impl StaticBendControlPoint {
    pub fn new(stiffness: f32, height: f32) -> Self {
        Self { stiffness, height }
    }
}

impl From<Vec2> for StaticBendControlPoint {
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}

impl Into<Vec2> for StaticBendControlPoint {
    fn into(self) -> Vec2 {
        Vec2::new(self.stiffness, self.height)
    }
}

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
    ///
    /// Controls how much the gradient overrides the base instance color.
    /// - `0.0`: The gradient is invisible.
    /// - `1.0`: The gradient is fully opaque.
    /// - `0.5`: A 50/50 blend.
    ///
    /// Defaults to `0.5`.
    pub tint: f32,
    /// The height (0.0 to 1.0) where the bottom color stops being solid
    /// and the gradient begins transitioning to be top-colored.
    ///
    /// Defaults to `0.2`.
    pub start: f32,
    /// The height (0.0 to 1.0) where the gradient finishes, becoming fully top-colored.
    ///
    /// Defaults to `0.8`.
    pub end: f32,
}

impl InstanceColorGradient {
    pub fn new(top: impl Into<Color>, bottom: impl Into<Color>) -> Self {
        Self {
            top: top.into(),
            bottom: bottom.into(),
            tint: 0.5,
            start: 0.2,
            end: 0.8,
        }
    }
}
