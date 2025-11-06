pub mod components;
pub mod events;

pub use components::*;
pub use events::*;

use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

/// Trigger of the [`SpawnProtoTypes`] Event.
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

/// Collection of material settings defining shader behavior.
#[derive(Clone, Debug, Reflect, Copy, Default)]
pub struct MaterialOptions {
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

    /// See [`PointLights`].
    pub point_lights: bool,
    /// See [`WindAffected`].
    pub wind_affected: bool,
    /// See [`LowQuality`].
    pub low_quality: bool,
    /// See [`AnalyticalNormals`].
    pub analytical_normals: bool,
    /// See [`InstanceColor`].
    pub color: Option<Color>,
    /// See [`SubsurfaceScattering`].
    pub subsurface_scattering: bool,
    /// See [`StaticBendStrength`].
    pub static_bend_strength: f32,
}

/// Type alias for a tuple of optional material components.
///
/// Used to construct or merge [`MaterialOptions`] from entity components.
pub type MaterialOptionData<'w> = (
    Option<&'w EnableDebug>,
    Option<&'w EnableBillboarding>,
    Option<&'w EdgeCorrectionFactor>,
    Option<&'w CurveFactor>,
    Option<&'w WindAffected>,
    Option<&'w LowQuality>,
    Option<&'w SubsurfaceScattering>,
    Option<&'w InstanceColor>,
    Option<&'w FastNormals>,
    Option<&'w StaticBendStrength>,
    Option<&'w AnalyticalNormals>,
    Option<&'w PointLights>,
);

impl From<MaterialOptionData<'_>> for MaterialOptions {
    fn from(
        (
            enable_debug,
            enable_billboarding,
            edge_correction_factor,
            curve_factor,
            wind_affected,
            low_q,
            subsurface_scattering,
            scatter_material_color,
            fast_normals,
            static_bend_strength,
            analytical_normals,
            point_lights,
        ): MaterialOptionData,
    ) -> Self {
        Self {
            debug: enable_debug.is_some(),
            enable_billboarding: enable_billboarding.is_some(),
            edge_correction_factor: edge_correction_factor.map(|x| **x).unwrap_or(0.),
            curve_factor: curve_factor.map(|x| **x).unwrap_or(0.),
            wind_affected: wind_affected.is_some(),
            low_quality: low_q.is_some(),
            subsurface_scattering: subsurface_scattering.is_some(),
            color: scatter_material_color.map(|x| **x),
            fast_normals: fast_normals.is_some(),
            static_bend_strength: static_bend_strength.map(|x| **x).unwrap_or(0.),
            analytical_normals: analytical_normals.is_some(),
            point_lights: point_lights.is_some(),
            ..default()
        }
    }
}

impl MaterialOptions {
    /// Merges [`MaterialOptionData`] into existing `MaterialOptions`.
    pub fn with(
        &self,
        (
            enable_debug,
            enable_billboarding,
            edge_correction_factor,
            curve_factor,
            wind_affected,
            low_q,
            subsurface_scattering,
            scatter_material_color,
            fast_normals,
            static_bend_strength,
            analytical_normals,
            point_lights,
        ): MaterialOptionData,
    ) -> Self {
        Self {
            debug: enable_debug.is_some() || self.debug,
            enable_billboarding: enable_billboarding.is_some() || self.enable_billboarding,
            edge_correction_factor: edge_correction_factor
                .map(|x| **x)
                .unwrap_or(self.edge_correction_factor),
            curve_factor: curve_factor.map(|x| **x).unwrap_or(self.curve_factor),
            wind_affected: wind_affected.is_some() || self.wind_affected,
            low_quality: low_q.is_some() || self.low_quality,
            subsurface_scattering: subsurface_scattering.is_some() || self.subsurface_scattering,
            color: if scatter_material_color.is_some() {
                scatter_material_color.map(|x| **x)
            } else {
                self.color
            },
            fast_normals: fast_normals.is_some() || self.fast_normals,
            static_bend_strength: static_bend_strength
                .map(|x| **x)
                .unwrap_or(self.static_bend_strength),
            analytical_normals: analytical_normals.is_some() || self.analytical_normals,
            point_lights: point_lights.is_some() || self.point_lights,
            ..*self
        }
    }

    /// Merges another [`MaterialOptions`] into this one.
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
        self.color = if other.color.is_some() {
            other.color
        } else {
            self.color
        };
        self.fast_normals = other.fast_normals || self.fast_normals;
        self.static_bend_strength = if other.static_bend_strength > 0. {
            other.static_bend_strength
        } else {
            self.static_bend_strength
        };
        self.analytical_normals = other.analytical_normals || self.analytical_normals;
        self.point_lights = other.point_lights || self.point_lights;
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
    /// Returns the [`MaterialOptions`].
    fn material_options(&self) -> &MaterialOptions;
}

/// Trait for sampling a value (e.g., density) at a world position.
pub trait Sampler {
    /// Samples the underlying data at `world_pos`.
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::palettes::css::*;

    #[test]
    fn test_from_material_option_data_all_some() {
        let debug = EnableDebug;
        let billboarding = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.1);
        let curve = CurveFactor(2.2);
        let wind = WindAffected;
        let low_q = LowQuality;
        let sss = SubsurfaceScattering;
        let color = InstanceColor(RED.into());
        let fast_normals = FastNormals;
        let bend = StaticBendStrength(3.3);
        let analytical_normals = AnalyticalNormals;
        let point_lights = PointLights;

        let data_all_some: MaterialOptionData = (
            Some(&debug),
            Some(&billboarding),
            Some(&edge),
            Some(&curve),
            Some(&wind),
            Some(&low_q),
            Some(&sss),
            Some(&color),
            Some(&fast_normals),
            Some(&bend),
            Some(&analytical_normals),
            Some(&point_lights),
        );

        let opts = MaterialOptions::from(data_all_some);

        assert_eq!(opts.debug, true);
        assert_eq!(opts.enable_billboarding, true);
        assert_eq!(opts.edge_correction_factor, 1.1);
        assert_eq!(opts.curve_factor, 2.2);
        assert_eq!(opts.wind_affected, true);
        assert_eq!(opts.low_quality, true);
        assert_eq!(opts.subsurface_scattering, true);
        assert_eq!(opts.color, Some(RED.into()));
        assert_eq!(opts.fast_normals, true);
        assert_eq!(opts.static_bend_strength, 3.3);
        assert_eq!(opts.analytical_normals, true);
        assert_eq!(opts.point_lights, true);

        assert_eq!(opts.controlled, false);
        assert_eq!(opts.debug_color, Color::default());
    }

    #[test]
    fn test_from_material_option_data_all_none_should_default() {
        let data_none: MaterialOptionData = (
            None, None, None, None, None, None, None, None, None, None, None, None,
        );
        let opts_none = MaterialOptions::from(data_none);

        let default_opts = MaterialOptions::default();
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
        assert_eq!(opts_none.color, default_opts.color);
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
        assert_eq!(opts_none.point_lights, default_opts.point_lights);
    }

    #[test]
    fn test_with_data_should_merge_and_override() {
        let base_opts = MaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            color: Some(BLUE.into()),
            curve_factor: 9.9,
            wind_affected: false,
            ..default()
        };

        let billboarding = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.5); // Override
        let color = InstanceColor(RED.into()); // Override
        let wind = WindAffected; // Merge

        let data: MaterialOptionData = (
            None,                // debug: None || true -> true
            Some(&billboarding), // billboarding: Some || false -> true
            Some(&edge),         // edge: Some(1.5) -> 1.5
            None,                // curve: None -> 9.9 (from base)
            Some(&wind),         // wind: Some || false -> true
            None,                // low_q
            None,                // sss
            Some(&color),        // color: Some(RED) -> RED
            None,                // fast_normals
            None,                // static_bend
            None,                // analytical_normals
            None,                // point_lights
        );

        let merged_opts = base_opts.with(data);

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
            merged_opts.color,
            Some(RED.into()),
            "Color should be overridden"
        );
    }

    #[test]
    fn test_with_options_should_merge_and_override() {
        let base = MaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            color: Some(BLUE.into()),
            static_bend_strength: 8.0,
            ..default()
        };

        let other = MaterialOptions {
            enable_billboarding: true,   // Merge: true
            edge_correction_factor: 1.0, // Override: 1.0 (since > 0)
            color: Some(RED.into()),     // Override
            static_bend_strength: 0.0,   // Keep base (since not > 0)
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
        assert_eq!(merged.color, Some(RED.into()), "Color should be overridden");

        let base_with_color = MaterialOptions {
            color: Some(BLUE.into()),
            ..default()
        };
        let other_no_color = MaterialOptions {
            color: None,
            ..default()
        };

        let merged_keep_color = base_with_color.with_options(other_no_color);

        assert_eq!(
            merged_keep_color.color,
            Some(BLUE.into()),
            "Should keep base color"
        );
    }

    #[test]
    fn test_builder_methods() {
        let opts = MaterialOptions::default()
            .with_controlled(true)
            .with_debug_color(GREEN.into());

        assert_eq!(opts.controlled, true);
        assert_eq!(opts.debug_color, GREEN.into());

        // default
        assert_eq!(opts.debug, false);
        assert_eq!(opts.edge_correction_factor, 0.0);
    }
}
