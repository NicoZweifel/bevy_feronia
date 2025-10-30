pub mod components;
pub mod events;

pub use components::*;
pub use events::*;

use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

/// Trigger of the [`SpawnProtoTypes`] Event.
/// Contains the scattered positions and information about the container as well as other scatter system relevant entities.
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
    fn test_from_material_option_data() {
        let data_none: MaterialOptionData = (
            None, None, None, None, None, None, None, None, None, None, None,
        );
        let opts_none = MaterialOptions::from(data_none);

        assert_eq!(opts_none.debug, false);
        assert_eq!(opts_none.enable_billboarding, false);
        assert_eq!(opts_none.edge_correction_factor, 0.0);
        assert_eq!(opts_none.curve_factor, 0.0);
        assert_eq!(opts_none.wind_affected, false);
        assert_eq!(opts_none.low_quality, false);
        assert_eq!(opts_none.subsurface_scattering, false);
        assert_eq!(opts_none.color, None);
        assert_eq!(opts_none.fast_normals, false);
        assert_eq!(opts_none.static_bend_strength, 0.0);
        assert_eq!(opts_none.analytical_normals, false);
        assert_eq!(opts_none.controlled, false);

        let debug = EnableDebug;
        let billow = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.5);
        let curve = CurveFactor(2.5);
        let wind = WindAffected;
        let low_q = LowQuality;
        let sss = SubsurfaceScattering;
        let color = InstanceColor(RED.into());
        let fast = FastNormals;
        let bend = StaticBendStrength(3.5);
        let analytical = AnalyticalNormals;

        let data_some: MaterialOptionData = (
            Some(&debug),
            Some(&billow),
            Some(&edge),
            Some(&curve),
            Some(&wind),
            Some(&low_q),
            Some(&sss),
            Some(&color),
            Some(&fast),
            Some(&bend),
            Some(&analytical),
        );
        let opts_some = MaterialOptions::from(data_some);

        assert_eq!(opts_some.debug, true);
        assert_eq!(opts_some.enable_billboarding, true);
        assert_eq!(opts_some.edge_correction_factor, 1.5);
        assert_eq!(opts_some.curve_factor, 2.5);
        assert_eq!(opts_some.wind_affected, true);
        assert_eq!(opts_some.low_quality, true);
        assert_eq!(opts_some.subsurface_scattering, true);
        assert_eq!(opts_some.color, Some(RED.into()));
        assert_eq!(opts_some.fast_normals, true);
        assert_eq!(opts_some.static_bend_strength, 3.5);
        assert_eq!(opts_some.analytical_normals, true);
    }

    #[test]
    fn test_with_method() {
        let base_opts = MaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            color: Some(BLUE.into()),
            ..default()
        };

        let billow = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.5);
        let color = InstanceColor(RED.into());

        let data: MaterialOptionData = (
            None, // debug
            Some(&billow),
            Some(&edge),
            None, // curve
            None, // wind
            None, // low_q
            None, // sss
            Some(&color),
            None, // fast_normals
            None, // static_bend
            None, // analytical_normals
        );

        let merged_opts = base_opts.with(data);

        assert_eq!(merged_opts.debug, true);
        assert_eq!(merged_opts.enable_billboarding, true);
        assert_eq!(merged_opts.edge_correction_factor, 1.5);
        assert_eq!(merged_opts.color, Some(RED.into()));
        assert_eq!(merged_opts.curve_factor, 0.0);
    }

    #[test]
    fn test_with_options() {
        let base = MaterialOptions {
            debug: true,
            edge_correction_factor: 5.0,
            color: Some(BLUE.into()),
            static_bend_strength: 8.0,
            ..default()
        };

        let other = MaterialOptions {
            enable_billboarding: true,
            edge_correction_factor: 1.0,
            color: Some(RED.into()),
            static_bend_strength: 0.0,
            ..default()
        };

        let merged = base.with_options(other);

        assert_eq!(merged.debug, true);
        assert_eq!(merged.enable_billboarding, true);

        assert_eq!(merged.edge_correction_factor, 1.0);
        assert_eq!(merged.static_bend_strength, 8.0);

        assert_eq!(merged.color, Some(RED.into()));
    }

    #[test]
    fn test_builder_methods() {
        let opts = MaterialOptions::default()
            .with_controlled(true)
            .with_debug_color(GREEN.into());

        assert_eq!(opts.controlled, true);
        assert_eq!(opts.debug_color, GREEN.into());
        assert_eq!(opts.debug, false);
    }
}
