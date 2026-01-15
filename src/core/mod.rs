pub mod components;
pub mod events;
pub mod utils;

pub use components::*;
pub use utils::*;

use crate::prelude::*;

use bevy_asset::{Asset, Handle};
use bevy_eidolon::prelude::{GpuCullCompute, InstanceColor};

use bevy_camera::primitives::Aabb;
use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_math::{Vec2, Vec3};
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
#[derive(Clone, Debug, Reflect, Copy, Default, PartialEq)]
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
    pub base_color: Option<Color>,
    /// See [`InstanceColorGradient`].
    pub top_color: Option<Color>,
    /// See [`InstanceColorGradient`].
    pub bottom_color: Option<Color>,
    /// See [`InstanceColorGradient`].
    pub tint_factor: f32,
    /// See [`InstanceColorGradient`].
    pub gradient_start: f32,
    /// See [`InstanceColorGradient`].
    pub gradient_end: f32,
    /// See [`SubsurfaceScattering`].
    pub subsurface_scattering: bool,
    /// See [`SubsurfaceScatteringScale`].
    pub subsurface_scattering_scale: f32,
    /// See [`SubsurfaceScatteringIntensity`].
    pub subsurface_scattering_intensity: f32,
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

    /// See [`AmbientOcclusion`].
    pub ambient_occlusion: bool,

    /// See [`StaticBendStrength`].
    pub static_bend_strength: f32,

    /// See [`StaticBendDirection`].
    pub static_bend_direction: Vec2,

    /// See [`StaticBendControlPoint`].
    pub static_bend_control_point: Vec2,

    /// See [`StaticBendMinMax`].
    pub static_bend_min_max: Vec2,
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
    pub base_color: Option<&'static InstanceColor>,
    pub color_gradient: Option<&'static InstanceColorGradient>,
    pub fast_normals: Option<&'static FastNormals>,
    pub analytical_normals: Option<&'static AnalyticalNormals>,
    pub directional_lights: Option<&'static DirectionalLights>,
    pub point_lights: Option<&'static PointLights>,
    pub static_shadow: Option<&'static StaticShadow>,
    pub unlit: Option<&'static Unlit>,
    pub gpu_cull: Option<&'static GpuCullCompute>,
    pub translucency: Option<&'static Translucency>,
    pub specular_strength: Option<&'static SpecularStrength>,
    pub specular_power: Option<&'static SpecularPower>,
    pub ambient_occlusion: Option<&'static AmbientOcclusion>,
    pub static_bend: Option<&'static StaticBend>,
    pub static_bend_strength: Option<&'static StaticBendStrength>,
    pub static_bend_direction: Option<&'static StaticBendDirection>,
    pub static_bend_control_point: Option<&'static StaticBendControlPoint>,
    pub static_bend_min_max: Option<&'static StaticBendMinMax>,
}

pub type MaterialChangedFilter = Or<(
    Or<(
        Changed<EnableDebug>,
        Changed<EnableBillboarding>,
        Changed<EdgeCorrectionFactor>,
        Changed<CurveFactor>,
        Changed<WindAffected>,
        Changed<LowQuality>,
    )>,
    Or<(
        Changed<SubsurfaceScattering>,
        Changed<SubsurfaceScatteringScale>,
        Changed<SubsurfaceScatteringIntensity>,
        Changed<InstanceColor>,
        Changed<InstanceColorGradient>,
        Changed<FastNormals>,
    )>,
    Or<(
        Changed<AnalyticalNormals>,
        Changed<DirectionalLights>,
        Changed<PointLights>,
        Changed<StaticShadow>,
        Changed<Unlit>,
    )>,
    Or<(
        Changed<GpuCullCompute>,
        Changed<Translucency>,
        Changed<SpecularStrength>,
        Changed<SpecularPower>,
        Changed<AmbientOcclusion>,
    )>,
    Or<(
        Changed<StaticBend>,
        Changed<StaticBendStrength>,
        Changed<StaticBendDirection>,
        Changed<StaticBendControlPoint>,
        Changed<StaticBendMinMax>,
    )>,
)>;

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
            base_color: data.base_color.map(|c| **c),
            top_color: data.color_gradient.map(|g| g.top),
            bottom_color: data.color_gradient.map(|g| g.bottom),
            tint_factor: data.color_gradient.map(|g| g.tint).unwrap_or(0.),
            gradient_end: data.color_gradient.map(|g| g.end).unwrap_or(0.),
            gradient_start: data.color_gradient.map(|g| g.start).unwrap_or(0.),
            fast_normals: data.fast_normals.is_some(),
            analytical_normals: data.analytical_normals.is_some(),
            point_lights: data.point_lights.is_some(),
            directional_lights: data.directional_lights.is_some(),
            static_shadows: data.static_shadow.is_some(),
            unlit: data.unlit.is_some(),
            gpu_cull: data.gpu_cull.is_some(),
            translucency: data.translucency.map(|s| **s).unwrap_or(0.),
            specular_strength: data.specular_strength.map(|s| **s).unwrap_or(0.),
            specular_power: data.specular_power.map(|s| **s).unwrap_or(0.),
            ambient_occlusion: data.ambient_occlusion.is_some(),
            static_bend_strength: data.static_bend_strength.map(|s| **s).unwrap_or(0.),
            static_bend_direction: data
                .static_bend_direction
                .map(|b| **b)
                .unwrap_or(Vec2::ZERO),
            static_bend_control_point: data
                .static_bend_control_point
                .cloned()
                .map(|b| b.into())
                .unwrap_or(Vec2::ZERO),
            static_bend_min_max: data
                .static_bend_min_max
                .cloned()
                .map(|b| b.into())
                .unwrap_or(Vec2::ZERO),
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
            base_color: data.base_color.map(|b| **b).or(self.base_color),
            top_color: data.color_gradient.map(|c| c.top).or(self.top_color),
            bottom_color: data.color_gradient.map(|c| c.bottom).or(self.bottom_color),
            tint_factor: data
                .color_gradient
                .map(|c| c.tint)
                .unwrap_or(self.tint_factor),
            gradient_start: data
                .color_gradient
                .map(|c| c.start)
                .unwrap_or(self.gradient_start),
            gradient_end: data
                .color_gradient
                .map(|c| c.end)
                .unwrap_or(self.gradient_end),
            fast_normals: data.fast_normals.is_some() || self.fast_normals,
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
            static_bend_strength: data
                .static_bend_strength
                .map(|s| **s)
                .unwrap_or(self.static_bend_strength),
            ambient_occlusion: data.ambient_occlusion.is_some() || self.ambient_occlusion,
            static_bend_direction: data
                .static_bend_direction
                .map(|b| **b)
                .unwrap_or(self.static_bend_direction),
            static_bend_control_point: data
                .static_bend_control_point
                .cloned()
                .map(|b| b.into())
                .unwrap_or(self.static_bend_control_point),
            static_bend_min_max: data
                .static_bend_min_max
                .cloned()
                .map(|b| b.into())
                .unwrap_or(self.static_bend_min_max),
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
        self.base_color = other.base_color.or(self.base_color);
        self.top_color = other.top_color.or(self.top_color);
        self.bottom_color = other.bottom_color.or(self.bottom_color);
        self.tint_factor = if other.tint_factor > 0. {
            other.tint_factor
        } else {
            self.tint_factor
        };
        self.gradient_start = if other.gradient_start > 0. {
            other.gradient_start
        } else {
            self.gradient_start
        };
        self.gradient_end = if other.gradient_end > 0. {
            other.gradient_end
        } else {
            self.gradient_end
        };
        self.fast_normals = other.fast_normals || self.fast_normals;
        self.analytical_normals = other.analytical_normals || self.analytical_normals;
        self.point_lights = other.point_lights || self.point_lights;
        self.directional_lights = other.directional_lights || self.directional_lights;
        self.static_shadows = other.static_shadows || self.static_shadows;
        self.unlit = other.unlit || self.unlit;
        self.gpu_cull = other.gpu_cull || self.gpu_cull;
        self.translucency = if other.translucency > 0. {
            other.translucency
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
        self.ambient_occlusion = other.ambient_occlusion || self.ambient_occlusion;
        self.static_bend_strength = if other.static_bend_strength > 0. {
            other.static_bend_strength
        } else {
            self.static_bend_strength
        };
        self.static_bend_direction = if other.static_bend_direction != Vec2::ZERO {
            other.static_bend_direction
        } else {
            self.static_bend_direction
        };
        self.static_bend_control_point = if other.static_bend_control_point != Vec2::ZERO {
            other.static_bend_control_point
        } else {
            self.static_bend_control_point
        };
        self.static_bend_min_max = if other.static_bend_min_max != Vec2::ZERO {
            other.static_bend_min_max
        } else {
            self.static_bend_min_max
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

    fn get_base_options() -> ScatterMaterialOptions {
        ScatterMaterialOptions {
            controlled: true,
            debug_color: RED.into(),
            debug: false,
            enable_billboarding: false,
            edge_correction_factor: 10.0,
            curve_factor: 10.0,
            wind_affected: false,
            low_quality: false,
            subsurface_scattering: false,
            subsurface_scattering_scale: 10.0,
            subsurface_scattering_intensity: 10.0,
            base_color: Some(BLACK.into()),
            top_color: Some(BLACK.into()),
            bottom_color: Some(BLACK.into()),
            tint_factor: 10.0,
            gradient_start: 10.0,
            gradient_end: 10.0,
            fast_normals: false,
            analytical_normals: false,
            directional_lights: false,
            point_lights: false,
            static_shadows: false,
            unlit: false,
            gpu_cull: false,
            translucency: 10.0,
            specular_strength: 10.0,
            specular_power: 10.0,
            ambient_occlusion: false,
            static_bend_strength: 10.0,
            static_bend_direction: Vec2::new(2., 4.),
            static_bend_control_point: Vec2::new(2., 4.),
            static_bend_min_max: Vec2::new(2., 4.),
        }
    }

    #[test]
    fn test_from_data_all_some_should_set_all_fields() {
        let debug = EnableDebug;
        let billboarding = EnableBillboarding;
        let edge = EdgeCorrectionFactor(1.1);
        let curve = CurveFactor(2.2);
        let wind = WindAffected;
        let low_q = LowQuality;
        let sss = SubsurfaceScattering;
        let sss_scale = SubsurfaceScatteringScale(0.5);
        let sss_intensity = SubsurfaceScatteringIntensity(0.2);
        let base_color = InstanceColor(GREEN.into());
        let color_gradient = InstanceColorGradient {
            top: RED.into(),
            bottom: BLUE.into(),
            tint: 1.0,
            start: 0.1,
            end: 0.9,
        };
        let fast_normals = FastNormals;
        let analytical_normals = AnalyticalNormals;
        let directional_lights = DirectionalLights;
        let point_lights = PointLights;
        let static_shadow = StaticShadow;
        let unlit = Unlit;
        let gpu_cull = GpuCullCompute;
        let translucency = Translucency(0.3);
        let specular_strength = SpecularStrength(0.5);
        let specular_power = SpecularPower(0.5);
        let ao = AmbientOcclusion;
        let bend = StaticBend;
        let bend_str = StaticBendStrength(3.3);
        let bend_dir = StaticBendDirection::new(1., 2.);
        let bend_cp = StaticBendControlPoint::new(1., 2.);
        let bend_min_max = StaticBendMinMax::new(1., 2.);

        let data = MaterialOptionDataItem {
            enable_debug: Some(&debug),
            enable_billboarding: Some(&billboarding),
            edge_correction_factor: Some(&edge),
            curve_factor: Some(&curve),
            wind_affected: Some(&wind),
            low_q: Some(&low_q),
            sss: Some(&sss),
            sss_scale: Some(&sss_scale),
            sss_intensity: Some(&sss_intensity),
            base_color: Some(&base_color),
            color_gradient: Some(&color_gradient),
            fast_normals: Some(&fast_normals),
            analytical_normals: Some(&analytical_normals),
            directional_lights: Some(&directional_lights),
            point_lights: Some(&point_lights),
            static_shadow: Some(&static_shadow),
            unlit: Some(&unlit),
            gpu_cull: Some(&gpu_cull),
            translucency: Some(&translucency),
            specular_strength: Some(&specular_strength),
            specular_power: Some(&specular_power),
            ambient_occlusion: Some(&ao),
            static_bend: Some(&bend),
            static_bend_strength: Some(&bend_str),
            static_bend_direction: Some(&bend_dir),
            static_bend_control_point: Some(&bend_cp),
            static_bend_min_max: Some(&bend_min_max),
        };

        // Act
        let result = ScatterMaterialOptions::from(data);

        let expected = ScatterMaterialOptions {
            // defaults
            controlled: false,
            debug_color: Color::default(),

            // changed
            debug: true,
            enable_billboarding: true,
            edge_correction_factor: 1.1,
            curve_factor: 2.2,
            wind_affected: true,
            low_quality: true,
            subsurface_scattering: true,
            subsurface_scattering_scale: 0.5,
            subsurface_scattering_intensity: 0.2,
            base_color: Some(GREEN.into()),
            top_color: Some(RED.into()),
            bottom_color: Some(BLUE.into()),
            tint_factor: 1.0,
            gradient_start: 0.1,
            gradient_end: 0.9,
            fast_normals: true,
            analytical_normals: true,
            directional_lights: true,
            point_lights: true,
            static_shadows: true,
            unlit: true,
            gpu_cull: true,
            translucency: 0.3,
            specular_strength: 0.5,
            specular_power: 0.5,
            ambient_occlusion: true,
            static_bend_strength: 3.3,
            static_bend_direction: Vec2::new(1., 2.),
            static_bend_control_point: Vec2::new(1., 2.),
            static_bend_min_max: Vec2::new(1., 2.),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_data_all_none_should_be_default() {
        // Arrange
        let data = MaterialOptionDataItem {
            enable_debug: None,
            enable_billboarding: None,
            edge_correction_factor: None,
            curve_factor: None,
            wind_affected: None,
            low_q: None,
            sss: None,
            sss_scale: None,
            sss_intensity: None,
            base_color: None,
            color_gradient: None,
            fast_normals: None,
            analytical_normals: None,
            directional_lights: None,
            point_lights: None,
            static_shadow: None,
            unlit: None,
            gpu_cull: None,
            translucency: None,
            specular_strength: None,
            specular_power: None,
            ambient_occlusion: None,
            static_bend: None,
            static_bend_strength: None,
            static_bend_direction: None,
            static_bend_control_point: None,
            static_bend_min_max: None,
        };

        // Act
        let result = ScatterMaterialOptions::from(data);

        // Assert
        assert_eq!(result, ScatterMaterialOptions::default());
    }

    #[test]
    fn test_with_data_full_override() {
        // Arrange
        let base = get_base_options();

        let debug = EnableDebug;
        let edge = EdgeCorrectionFactor(0.5);
        let col = InstanceColor(WHITE.into());
        let bb = EnableBillboarding;
        let cur = CurveFactor(1.0);
        let wnd = WindAffected;
        let lq = LowQuality;
        let ss = SubsurfaceScattering;
        let sss_s = SubsurfaceScatteringScale(1.0);
        let sss_i = SubsurfaceScatteringIntensity(1.0);
        let grad = InstanceColorGradient {
            top: BLUE.into(),
            bottom: GREEN.into(),
            tint: 0.5,
            start: 0.1,
            end: 0.9,
        };
        let fnorm = FastNormals;
        let anorm = AnalyticalNormals;
        let dl = DirectionalLights;
        let pl = PointLights;
        let shad = StaticShadow;
        let unl = Unlit;
        let cull = GpuCullCompute;
        let trans = Translucency(1.0);
        let spec_s = SpecularStrength(1.0);
        let spec_p = SpecularPower(1.0);
        let ao = AmbientOcclusion;
        let bend = StaticBend;
        let bend_str = StaticBendStrength(1.0);
        let bend_dir = StaticBendDirection::new(1., 2.);
        let bend_cp = StaticBendControlPoint::new(1., 2.);
        let bend_min_max = StaticBendMinMax::new(1., 2.);

        let data = MaterialOptionDataItem {
            enable_debug: Some(&debug),
            enable_billboarding: Some(&bb),
            edge_correction_factor: Some(&edge),
            curve_factor: Some(&cur),
            wind_affected: Some(&wnd),
            low_q: Some(&lq),
            sss: Some(&ss),
            sss_scale: Some(&sss_s),
            sss_intensity: Some(&sss_i),
            base_color: Some(&col),
            color_gradient: Some(&grad),
            fast_normals: Some(&fnorm),
            analytical_normals: Some(&anorm),
            directional_lights: Some(&dl),
            point_lights: Some(&pl),
            static_shadow: Some(&shad),
            unlit: Some(&unl),
            gpu_cull: Some(&cull),
            translucency: Some(&trans),
            specular_strength: Some(&spec_s),
            specular_power: Some(&spec_p),
            ambient_occlusion: Some(&ao),
            static_bend: Some(&bend),
            static_bend_strength: Some(&bend_str),
            static_bend_direction: Some(&bend_dir),
            static_bend_control_point: Some(&bend_cp),
            static_bend_min_max: Some(&bend_min_max),
        };

        // Act
        let result = base.with(data);

        // Assert
        let expected = ScatterMaterialOptions {
            // Preserve
            controlled: true,
            debug_color: RED.into(),

            // Override
            debug: true,
            enable_billboarding: true,
            edge_correction_factor: 0.5,
            curve_factor: 1.0,
            wind_affected: true,
            low_quality: true,
            subsurface_scattering: true,
            subsurface_scattering_scale: 1.0,
            subsurface_scattering_intensity: 1.0,
            base_color: Some(WHITE.into()),
            top_color: Some(BLUE.into()),
            bottom_color: Some(GREEN.into()),
            tint_factor: 0.5,
            gradient_start: 0.1,
            gradient_end: 0.9,
            fast_normals: true,
            analytical_normals: true,
            directional_lights: true,
            point_lights: true,
            static_shadows: true,
            unlit: true,
            gpu_cull: true,
            translucency: 1.0,
            specular_strength: 1.0,
            specular_power: 1.0,
            ambient_occlusion: true,
            static_bend_strength: 1.0,
            static_bend_direction: Vec2::new(1., 2.),
            static_bend_control_point: Vec2::new(1., 2.),
            static_bend_min_max: Vec2::new(1., 2.),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_data_preserve_if_none() {
        // Arrange
        let base = get_base_options();
        let data_empty = MaterialOptionDataItem {
            enable_debug: None,
            enable_billboarding: None,
            edge_correction_factor: None,
            curve_factor: None,
            wind_affected: None,
            low_q: None,
            sss: None,
            sss_scale: None,
            sss_intensity: None,
            base_color: None,
            color_gradient: None,
            fast_normals: None,
            analytical_normals: None,
            directional_lights: None,
            point_lights: None,
            static_shadow: None,
            unlit: None,
            gpu_cull: None,
            translucency: None,
            specular_strength: None,
            specular_power: None,
            ambient_occlusion: None,
            static_bend: None,
            static_bend_strength: None,
            static_bend_direction: None,
            static_bend_control_point: None,
            static_bend_min_max: None,
        };

        // Act
        let result = base.with(data_empty);

        // Assert
        let expected = base;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_options_merge_logic() {
        // Arrange
        let base = get_base_options();
        let other = ScatterMaterialOptions {
            enable_billboarding: true,     // True (OR)
            edge_correction_factor: 0.5,   // Override
            static_bend_strength: 0.0,     // Preserve
            top_color: Some(WHITE.into()), // Override
            ..default()
        };

        // Act
        let result = base.with_options(other);

        // Assert
        let expected = ScatterMaterialOptions {
            // OR
            enable_billboarding: true,

            // Preserve
            debug: false,
            curve_factor: 10.0,
            translucency: 10.0,
            base_color: Some(BLACK.into()),

            // Override
            edge_correction_factor: 0.5,
            top_color: Some(WHITE.into()),

            // Base
            controlled: true,
            debug_color: RED.into(),
            wind_affected: false,
            low_quality: false,
            subsurface_scattering: false,
            subsurface_scattering_scale: 10.0,
            subsurface_scattering_intensity: 10.0,
            bottom_color: Some(BLACK.into()),
            tint_factor: 10.0,
            gradient_start: 10.0,
            gradient_end: 10.0,
            fast_normals: false,
            analytical_normals: false,
            directional_lights: false,
            point_lights: false,
            static_shadows: false,
            unlit: false,
            gpu_cull: false,
            specular_strength: 10.0,
            specular_power: 10.0,
            ambient_occlusion: false,
            static_bend_strength: 10.0,
            static_bend_direction: Vec2::new(2., 4.),
            static_bend_control_point: Vec2::new(2., 4.),
            static_bend_min_max: Vec2::new(2., 4.),
        };

        assert_eq!(result, expected);
    }
}
