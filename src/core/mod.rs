pub mod components;
pub mod events;

use bevy_asset::{Asset, Handle};
pub use components::*;

use crate::prelude::*;
use bevy_camera::primitives::Aabb;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_math::Vec3;
use bevy_mesh::Mesh;
use bevy_reflect::Reflect;
use bevy_utils::default;

/// Trigger of the [`SpawnScatterAssets`] Event.
/// Contains the scattered positions and contextual information (like `layer`, `chunk`, `root`).
#[derive(Clone, Debug)]
pub struct SpawnTrigger {
    /// [`Chunk`] this spawn is associated with, if any.
    pub chunk: Option<Entity>,
    /// [`ScatterLayer`] entity this spawn belongs to.
    pub layer: Entity,
    /// [`ScatterRoot`] entity this spawn belongs to.
    pub root: Entity,
    /// Target entity that triggered this spawn (e.g., the chunk or layer).
    pub target: Entity,
    /// Computed scatter results.
    pub data: Vec<ScatterResult>,
    /// Seed used for deterministic scattering.
    pub seed: u64,
}

impl<T> From<On<'_, '_, ScatterResults<T>>> for SpawnTrigger
where
    T: ScatterMaterial,
{
    fn from(value: On<ScatterResults<T>>) -> Self {
        Self {
            chunk: value.chunk,
            layer: value.layer,
            target: value.entity,
            data: value.data.clone(),
            root: value.root,
            seed: value.seed,
        }
    }
}

/// Collection of material settings defining shader behavior.
#[derive(Clone, Debug, Reflect, Copy, Default)]
pub struct ScatterMaterialOptions {
    /// If true, material is not automatically synced with global [`Wind`].
    // TODO fix this und update options/material settings properly
    pub controlled: bool,
    /// Color to use when `debug` is enabled.
    pub debug_color: Color,
    /// See [`EnableDebug`].
    pub debug: bool,
    /// See [`EnableBillboarding`].
    pub enable_billboarding: bool,
    /// See [`FastNormals`].
    pub fast_normals: bool,
    /// See [`EdgeCorrectionFactor`].
    pub edge_correction_factor: f32,
    /// See [`CurveFactor`].
    pub curve_factor: f32,
    /// See [`DirectionalLights`].
    pub directional_lights: bool,
    /// See [`PointLights`].
    pub point_lights: bool,
    /// See [`WindAffected`].
    pub wind_affected: bool,
    /// See [`LowQuality`].
    pub low_quality: bool,
    /// See [`AnalyticalNormals`].
    pub analytical_normals: bool,
    /// See [`InstanceColor`].
    pub top_color: Option<Color>,
    /// See [`InstanceColor`].
    pub bottom_color: Option<Color>,
    /// See [`SubsurfaceScattering`].
    pub subsurface_scattering: bool,
    /// See [`SubsurfaceScatteringScale`].
    pub subsurface_scattering_scale: f32,
    /// See [`SubsurfaceScatteringIntensity`].
    pub subsurface_scattering_intensity: f32,
    /// See [`StaticBendStrength`].
    pub static_bend_strength: f32,
    /// See [`StaticShadow`].
    pub static_shadows: bool,
    /// See [`Unlit`].
    pub unlit: bool,
    /// See [`GpuCull`].
    pub gpu_cull: bool,

    /// See [`Translucency`].
    pub translucency: f32,
    /// See [`SpecularStrength`].
    pub specular_strength: f32,
    /// See [`SpecularPower`].
    pub specular_power: f32,
}

/// Collection of optional material components, usable as `QueryData`.
#[derive(QueryData)]
#[query_data(derive(Clone, Copy))]
pub struct MaterialOptionData {
    pub enable_debug: Option<&'static EnableDebug>,
    pub enable_billboarding: Option<&'static EnableBillboarding>,
    pub edge_correction_factor: Option<&'static EdgeCorrectionFactor>,
    pub curve_factor: Option<&'static CurveFactor>,
    pub wind_affected: Option<&'static WindAffected>,
    pub low_q: Option<&'static LowQuality>,
    pub sss: Option<&'static SubsurfaceScattering>,
    pub sss_scale: Option<&'static SubsurfaceScatteringScale>,
    pub sss_intensity: Option<&'static SubsurfaceScatteringIntensity>,
    pub scatter_material_color: Option<&'static InstanceColor>,
    pub fast_normals: Option<&'static FastNormals>,
    pub static_bend_strength: Option<&'static StaticBendStrength>,
    pub analytical_normals: Option<&'static AnalyticalNormals>,
    pub directional_lights: Option<&'static DirectionalLights>,
    pub point_lights: Option<&'static PointLights>,
    pub static_shadow: Option<&'static StaticShadow>,
    pub unlit: Option<&'static Unlit>,
    pub gpu_cull: Option<&'static GpuCull>,
    pub translucency: Option<&'static Translucency>,
    pub specular_strength: Option<&'static SpecularStrength>,
    pub specular_power: Option<&'static SpecularPower>,
}

impl From<MaterialOptionDataItem<'_, '_>> for ScatterMaterialOptions {
    fn from(data: MaterialOptionDataItem<'_, '_>) -> Self {
        Self {
            debug: data.enable_debug.is_some(),
            enable_billboarding: data.enable_billboarding.is_some(),
            edge_correction_factor: data.edge_correction_factor.map(|e| **e).unwrap_or(0.),
            curve_factor: data.curve_factor.map(|c| **c).unwrap_or(0.),
            wind_affected: data.wind_affected.is_some(),
            low_quality: data.low_q.is_some(),
            subsurface_scattering: data.sss.is_some(),
            subsurface_scattering_intensity: data.sss_intensity.map(|s| **s).unwrap_or(0.),
            subsurface_scattering_scale: data.sss_scale.map(|s| **s).unwrap_or(0.),
            top_color: data.scatter_material_color.map(|s| s.top),
            bottom_color: data.scatter_material_color.map(|s| s.bottom),
            fast_normals: data.fast_normals.is_some(),
            static_bend_strength: data.static_bend_strength.map(|s| **s).unwrap_or(0.),
            analytical_normals: data.analytical_normals.is_some(),
            point_lights: data.point_lights.is_some(),
            directional_lights: data.directional_lights.is_some(),
            static_shadows: data.static_shadow.is_some(),
            unlit: data.unlit.is_some(),
            gpu_cull: data.gpu_cull.is_some(),
            translucency: data.translucency.map(|s| **s).unwrap_or(0.),
            specular_strength: data.specular_strength.map(|s| **s).unwrap_or(0.),
            specular_power: data.specular_power.map(|s| **s).unwrap_or(0.),
            ..default()
        }
    }
}

impl ScatterMaterialOptions {
    /// Merges [`MaterialOptionDataItem`] into existing `MaterialOptions`.
    pub fn with(&self, data: MaterialOptionDataItem) -> Self {
        Self {
            debug: data.enable_debug.is_some() || self.debug,
            enable_billboarding: data.enable_billboarding.is_some() || self.enable_billboarding,
            edge_correction_factor: data
                .edge_correction_factor
                .map(|f| **f)
                .unwrap_or(self.edge_correction_factor),
            curve_factor: data.curve_factor.map(|f| **f).unwrap_or(self.curve_factor),
            wind_affected: data.wind_affected.is_some() || self.wind_affected,
            low_quality: data.low_q.is_some() || self.low_quality,
            subsurface_scattering: data.sss.is_some() || self.subsurface_scattering,
            subsurface_scattering_scale: data
                .sss_scale
                .map(|s| **s)
                .unwrap_or(self.subsurface_scattering_scale),
            subsurface_scattering_intensity: data
                .sss_intensity
                .map(|s| **s)
                .unwrap_or(self.subsurface_scattering_intensity),
            top_color: data
                .scatter_material_color
                .map(|c| c.top)
                .or(self.top_color),
            bottom_color: data
                .scatter_material_color
                .map(|c| c.bottom)
                .or(self.bottom_color),
            fast_normals: data.fast_normals.is_some() || self.fast_normals,
            static_bend_strength: data
                .static_bend_strength
                .map(|s| **s)
                .unwrap_or(self.static_bend_strength),
            analytical_normals: data.analytical_normals.is_some() || self.analytical_normals,
            point_lights: data.point_lights.is_some() || self.point_lights,
            directional_lights: data.directional_lights.is_some() || self.directional_lights,
            static_shadows: data.static_shadow.is_some() || self.static_shadows,
            unlit: data.unlit.is_some() || self.unlit,
            gpu_cull: data.gpu_cull.is_some() || self.gpu_cull,
            translucency: data.translucency.map(|s| **s).unwrap_or(self.translucency),
            specular_strength: data
                .specular_strength
                .map(|s| **s)
                .unwrap_or(self.specular_strength),
            specular_power: data
                .specular_power
                .map(|s| **s)
                .unwrap_or(self.specular_power),
            ..*self
        }
    }

    /// Merges another [`ScatterMaterialOptions`] into this one.
    pub fn with_options(mut self, other: Self) -> Self {
        self.debug = other.debug || self.debug;
        self.enable_billboarding = other.enable_billboarding || self.enable_billboarding;
        self.edge_correction_factor = if other.edge_correction_factor > 0. {
            other.edge_correction_factor
        } else {
            self.edge_correction_factor
        };
        self.curve_factor = if other.curve_factor > 0. {
            other.curve_factor
        } else {
            self.curve_factor
        };
        self.wind_affected = other.wind_affected || self.wind_affected;
        self.low_quality = other.low_quality || self.low_quality;
        self.subsurface_scattering = other.subsurface_scattering || self.subsurface_scattering;
        self.top_color = other.top_color.or(self.top_color);
        self.bottom_color = other.bottom_color.or(self.bottom_color);
        self.fast_normals = other.fast_normals || self.fast_normals;
        self.static_bend_strength = if other.static_bend_strength > 0. {
            other.static_bend_strength
        } else {
            self.static_bend_strength
        };
        self.analytical_normals = other.analytical_normals || self.analytical_normals;
        self.point_lights = other.point_lights || self.point_lights;
        self.directional_lights = other.directional_lights || self.directional_lights;
        self.static_shadows = other.static_shadows || self.static_shadows;
        self.unlit = other.unlit || self.unlit;
        self.gpu_cull = other.gpu_cull || self.gpu_cull;
        self.translucency = if other.translucency > 0. {
            other.specular_power
        } else {
            self.translucency
        };
        self.specular_strength = if other.specular_strength > 0. {
            other.specular_strength
        } else {
            self.specular_strength
        };
        self.specular_power = if other.specular_power > 0. {
            other.specular_power
        } else {
            self.specular_power
        };
        self
    }

    pub fn with_debug_color(mut self, debug_color: Color) -> Self {
        self.debug_color = debug_color;
        self
    }

    pub fn with_controlled(mut self, controlled: bool) -> Self {
        self.controlled = controlled;
        self
    }
}

/// Trait defining the required properties for a spawnable prototype.
pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    /// Returns the [`Mesh`] handle.
    fn mesh(&self) -> &Handle<Mesh>;
    /// Returns the material handle.
    fn material(&self) -> &Handle<T>;
    /// Returns the [`Wind`] settings.
    fn wind(&self) -> &Wind;
    /// Returns the [`Aabb`] (Axis-Aligned Bounding Box).
    fn aabb(&self) -> &Aabb;
    /// Returns the [`LevelOfDetail`].
    fn lod(&self) -> &LevelOfDetail;
    /// Returns the [`ScatterMaterialOptions`].
    fn material_options(&self) -> &ScatterMaterialOptions;
}

/// Trait for sampling a value (e.g., density) at a world position.
pub trait Sampler {
    /// Samples the underlying data at `world_pos`.
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_color::palettes::css::*;

    #[test]
    fn test_from_material_option_data_all_some_should_set() {
        // Arrange
        let debug = EnableDebug;
        let billboarding = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.1);
        let curve = CurveFactor(2.2);
        let wind = WindAffected;
        let low_q = LowQuality;
        let sss = SubsurfaceScattering;
        let sss_scale = SubsurfaceScatteringScale(0.5);
        let sss_intensity = SubsurfaceScatteringIntensity(0.2);
        let color = InstanceColor::new(RED.into(), BLUE.into());
        let fast_normals = FastNormals;
        let bend = StaticBendStrength(3.3);
        let analytical_normals = AnalyticalNormals;
        let directional_lights = DirectionalLights;
        let point_lights = PointLights;
        let static_shadow = StaticShadow;
        let unlit = Unlit;
        let gpu_cull = GpuCull;
        let translucency = Translucency(0.3);
        let specular_strength = SpecularStrength(0.5);
        let specular_power = SpecularPower(0.5);

        let data_all_some = MaterialOptionDataItem {
            enable_debug: Some(&debug),
            enable_billboarding: Some(&billboarding),
            edge_correction_factor: Some(&edge),
            curve_factor: Some(&curve),
            wind_affected: Some(&wind),
            low_q: Some(&low_q),
            sss: Some(&sss),
            sss_scale: Some(&sss_scale),
            sss_intensity: Some(&sss_intensity),
            scatter_material_color: Some(&color),
            fast_normals: Some(&fast_normals),
            static_bend_strength: Some(&bend),
            analytical_normals: Some(&analytical_normals),
            directional_lights: Some(&directional_lights),
            point_lights: Some(&point_lights),
            static_shadow: Some(&static_shadow),
            unlit: Some(&unlit),
            gpu_cull: Some(&gpu_cull),
            translucency: Some(&translucency),
            specular_strength: Some(&specular_strength),
            specular_power: Some(&specular_power),
        };

        // Act
        let opts = ScatterMaterialOptions::from(data_all_some);

        // Assert
        assert_eq!(opts.debug, true);
        assert_eq!(opts.enable_billboarding, true);
        assert_eq!(opts.edge_correction_factor, 1.1);
        assert_eq!(opts.curve_factor, 2.2);
        assert_eq!(opts.wind_affected, true);
        assert_eq!(opts.low_quality, true);
        assert_eq!(opts.subsurface_scattering, true);
        assert_eq!(opts.top_color, Some(RED.into()));
        assert_eq!(opts.bottom_color, Some(BLUE.into()));
        assert_eq!(opts.fast_normals, true);
        assert_eq!(opts.static_bend_strength, 3.3);
        assert_eq!(opts.analytical_normals, true);
        assert_eq!(opts.point_lights, true);
        assert_eq!(opts.directional_lights, true);
        assert_eq!(opts.point_lights, true);
        assert_eq!(opts.static_shadows, true);
        assert_eq!(opts.unlit, true);
        assert_eq!(opts.controlled, false);
        assert_eq!(opts.debug_color, Color::default());
        assert_eq!(opts.gpu_cull, true);
        assert_eq!(opts.translucency, 0.3);
        assert_eq!(opts.specular_strength, 0.5);
        assert_eq!(opts.specular_power, 0.5);
    }

    #[test]
    fn test_from_material_option_data_all_none_should_default() {
        // Arrange
        let data_none = MaterialOptionDataItem {
            enable_debug: None,
            enable_billboarding: None,
            edge_correction_factor: None,
            curve_factor: None,
            wind_affected: None,
            low_q: None,
            sss: None,
            sss_scale: None,
            sss_intensity: None,
            scatter_material_color: None,
            fast_normals: None,
            static_bend_strength: None,
            analytical_normals: None,
            directional_lights: None,
            point_lights: None,
            static_shadow: None,
            unlit: None,
            gpu_cull: None,
            translucency: None,
            specular_strength: None,
            specular_power: None,
        };
        let default_opts = ScatterMaterialOptions::default();

        // Act
        let opts_none = ScatterMaterialOptions::from(data_none);

        // Assert
        assert_eq!(opts_none.debug, default_opts.debug);
        assert_eq!(
            opts_none.enable_billboarding,
            default_opts.enable_billboarding
        );
        assert_eq!(
            opts_none.edge_correction_factor,
            default_opts.edge_correction_factor
        );
        assert_eq!(opts_none.curve_factor, default_opts.curve_factor);
        assert_eq!(opts_none.wind_affected, default_opts.wind_affected);
        assert_eq!(opts_none.low_quality, default_opts.low_quality);
        assert_eq!(
            opts_none.subsurface_scattering,
            default_opts.subsurface_scattering
        );
        assert_eq!(
            opts_none.subsurface_scattering_scale,
            default_opts.subsurface_scattering_scale
        );
        assert_eq!(
            opts_none.subsurface_scattering_intensity,
            default_opts.subsurface_scattering_intensity
        );
        assert_eq!(opts_none.top_color, default_opts.top_color);
        assert_eq!(opts_none.bottom_color, default_opts.bottom_color);
        assert_eq!(opts_none.fast_normals, default_opts.fast_normals);
        assert_eq!(
            opts_none.static_bend_strength,
            default_opts.static_bend_strength
        );
        assert_eq!(
            opts_none.analytical_normals,
            default_opts.analytical_normals
        );
        assert_eq!(opts_none.controlled, default_opts.controlled);
        assert_eq!(opts_none.debug_color, default_opts.debug_color);
        assert_eq!(
            opts_none.directional_lights,
            default_opts.directional_lights
        );
        assert_eq!(opts_none.point_lights, default_opts.point_lights);
        assert_eq!(opts_none.static_shadows, default_opts.static_shadows);
        assert_eq!(opts_none.unlit, default_opts.unlit);
        assert_eq!(opts_none.gpu_cull, default_opts.gpu_cull);
        assert_eq!(opts_none.translucency, default_opts.translucency);
        assert_eq!(opts_none.specular_strength, default_opts.specular_strength);
        assert_eq!(opts_none.specular_power, default_opts.specular_power);
    }

    #[test]
    fn test_with_data_should_merge_and_override() {
        // Arrange
        let base_opts = ScatterMaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            top_color: Some(BLUE.into()),
            curve_factor: 9.9,
            wind_affected: false,
            ..default()
        };

        let billboarding = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.5); // Override
        let color = InstanceColor::new(RED.into(), default()); // Override
        let wind = WindAffected; // Merge

        let data = MaterialOptionDataItem {
            enable_debug: None,
            enable_billboarding: Some(&billboarding),
            edge_correction_factor: Some(&edge),
            curve_factor: None,
            wind_affected: Some(&wind),
            low_q: None,
            sss: None,
            sss_scale: None,
            sss_intensity: None,
            scatter_material_color: Some(&color),
            fast_normals: None,
            static_bend_strength: None,
            analytical_normals: None,
            directional_lights: None,
            point_lights: None,
            static_shadow: None,
            unlit: None,
            gpu_cull: None,
            translucency: None,
            specular_strength: None,
            specular_power: None,
        };

        // Act
        let merged_opts = base_opts.with(data);

        // Assert
        assert_eq!(merged_opts.debug, true, "Debug should be preserved");
        assert_eq!(
            merged_opts.enable_billboarding, true,
            "Billboarding should be set"
        );
        assert_eq!(
            merged_opts.edge_correction_factor, 1.5,
            "Edge should be overridden"
        );
        assert_eq!(
            merged_opts.curve_factor, 9.9,
            "Curve factor should be preserved"
        );
        assert_eq!(
            merged_opts.wind_affected, true,
            "Wind affected should merge"
        );
        assert_eq!(
            merged_opts.top_color,
            Some(RED.into()),
            "Color should be overridden"
        );
    }

    #[test]
    fn test_with_options_should_merge_and_override() {
        // Arrange
        let base = ScatterMaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            top_color: Some(BLUE.into()),
            bottom_color: Some(BLUE.into()),
            static_bend_strength: 8.0,
            ..default()
        };

        let other = ScatterMaterialOptions {
            enable_billboarding: true,        // Merge: true
            edge_correction_factor: 1.0,      // Override: 1.0 (since > 0)
            top_color: Some(RED.into()),      // Override
            bottom_color: Some(GREEN.into()), // Override
            static_bend_strength: 0.0,        // Keep base (since not > 0)
            ..default()
        };

        let merged = base.with_options(other);

        assert_eq!(merged.debug, true, "Debug should be preserved");
        assert_eq!(
            merged.enable_billboarding, true,
            "Billboarding should be merged"
        );
        assert_eq!(
            merged.edge_correction_factor, 1.0,
            "Edge factor should be overridden"
        );
        assert_eq!(
            merged.static_bend_strength, 8.0,
            "Bend strength should be preserved"
        );
        assert_eq!(
            merged.top_color,
            Some(RED.into()),
            "Color should be overridden"
        );
        assert_eq!(
            merged.bottom_color,
            Some(GREEN.into()),
            "Color should be overridden"
        );
    }

    #[test]
    fn test_with_options_should_keep_colors_when_no_options_provided() {
        // Arrange
        let base_with_color = ScatterMaterialOptions {
            top_color: Some(BLUE.into()),
            bottom_color: Some(RED.into()),
            ..default()
        };
        let other_no_color = ScatterMaterialOptions {
            top_color: None,
            bottom_color: None,
            ..default()
        };

        // Act
        let merged_keep_color = base_with_color.with_options(other_no_color);

        // Assert
        assert_eq!(
            merged_keep_color.top_color,
            Some(BLUE.into()),
            "Should keep base color"
        );

        assert_eq!(
            merged_keep_color.bottom_color,
            Some(RED.into()),
            "Should keep base color"
        );
    }

    #[test]
    fn test_builder_methods_should_set() {
        let opts = ScatterMaterialOptions::default()
            .with_controlled(true)
            .with_debug_color(GREEN.into());

        assert_eq!(opts.controlled, true);
        assert_eq!(opts.debug_color, GREEN.into());

        // default
        assert_eq!(opts.debug, false);
        assert_eq!(opts.edge_correction_factor, 0.0);
    }
}
