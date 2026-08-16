//! Types and traits used across the modules.
//!
//! Defines the contracts and protocols that the rest of the code uses.

pub mod components;
pub mod events;
pub mod options;

pub use components::*;
pub use events::*;
pub use options::prelude::*;

use crate::prelude::*;

use bevy_asset::{Asset, Handle};
use bevy_eidolon::prelude::{GpuCullCompute, InstanceColor};

use bevy_camera::primitives::Aabb;
use bevy_camera::visibility::RenderLayers;
use bevy_color::Color;
use bevy_ecs::{prelude::*, query::QueryData};
use bevy_math::Vec3;
use bevy_mesh::Mesh;
use bevy_reflect::Reflect;

/// Trigger of the [`SpawnScatterAssets`] Event.
#[derive(Clone, Debug)]
pub struct SpawnTrigger {
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    pub target: Entity,
    pub data: Vec<ScatterResult>,
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
#[derive(Clone, Debug, Reflect, Default, PartialEq)]
pub struct ScatterMaterialOptions {
    pub general: GeneralOptions,
    pub geometry: GeometryOptions,
    pub wind: WindOptions,
    pub bend: StaticBendOptions,
    pub color: ColorOptions,
    pub lighting: LightingOptions,
    pub render_layers: Option<RenderLayers>,
}

/// Collection of optional material components, usable as `QueryData`.
#[derive(QueryData)]
#[query_data(derive(Clone, Copy))]
pub struct MaterialOptionData {
    // General
    pub enable_debug: Option<&'static EnableDebug>,
    pub gpu_cull: Option<&'static GpuCullCompute>,
    pub disable_prepass: Option<&'static DisablePrepass>,

    // Geometry
    pub enable_billboarding: Option<&'static EnableBillboarding>,
    pub edge_correction_factor: Option<&'static EdgeCorrectionFactor>,
    pub wind_affected: Option<&'static WindAffected>,
    pub low_q: Option<&'static LowQuality>,

    // Static Bend
    pub static_bend: Option<&'static StaticBend>,
    pub static_bend_strength: Option<&'static StaticBendStrength>,
    pub static_bend_direction: Option<&'static StaticBendDirection>,
    pub static_bend_control_point: Option<&'static StaticBendControlPoint>,
    pub static_bend_min_max: Option<&'static StaticBendMinMax>,

    // Color
    pub base_color: Option<&'static InstanceColor>,
    pub color_gradient: Option<&'static InstanceColorGradient>,

    // Lighting
    pub curve_factor: Option<&'static CurveFactor>,
    pub fast_normals: Option<&'static FastNormals>,
    pub analytical_normals: Option<&'static AnalyticalNormals>,
    pub unlit: Option<&'static Unlit>,
    pub directional_lights: Option<&'static DirectionalLights>,
    pub point_lights: Option<&'static PointLights>,
    pub static_shadow: Option<&'static StaticShadow>,
    pub ambient_occlusion: Option<&'static AmbientOcclusion>,
    pub sss: Option<&'static SubsurfaceScattering>,
    pub sss_scale: Option<&'static SubsurfaceScatteringScale>,
    pub sss_intensity: Option<&'static SubsurfaceScatteringIntensity>,

    // PBR
    pub standard_pbr: Option<&'static StandardPbr>,
    pub roughness: Option<&'static Roughness>,
    pub metallic: Option<&'static Metallic>,
    pub reflectance: Option<&'static Reflectance>,

    // Blinn-Phong
    pub translucency: Option<&'static Translucency>,
    pub specular_strength: Option<&'static SpecularStrength>,
    pub specular_power: Option<&'static SpecularPower>,
    pub diffuse_scaling: Option<&'static DiffuseScaling>,
    pub light_intensity: Option<&'static LightIntensity>,
    pub ambient_light_intensity: Option<&'static AmbientLightIntensity>,

    pub render_layers: Option<&'static RenderLayers>,
}

pub type MaterialChangedFilter = Or<(
    Or<(
        Changed<EnableDebug>,
        Changed<GpuCullCompute>,
        Changed<DisablePrepass>,
        Changed<EnableBillboarding>,
        Changed<WindAffected>,
        Changed<LowQuality>,
        Changed<EdgeCorrectionFactor>,
    )>,
    Or<(
        Changed<StaticBend>,
        Changed<StaticBendStrength>,
        Changed<StaticBendDirection>,
        Changed<StaticBendControlPoint>,
        Changed<StaticBendMinMax>,
    )>,
    Or<(Changed<InstanceColor>, Changed<InstanceColorGradient>)>,
    Or<(
        Changed<CurveFactor>,
        Changed<FastNormals>,
        Changed<AnalyticalNormals>,
    )>,
    Or<(
        Changed<Unlit>,
        Changed<DirectionalLights>,
        Changed<PointLights>,
        Changed<StaticShadow>,
        Changed<AmbientOcclusion>,
        Changed<SubsurfaceScattering>,
        Changed<SubsurfaceScatteringScale>,
        Changed<SubsurfaceScatteringIntensity>,
    )>,
    Or<(
        Changed<StandardPbr>,
        Changed<Roughness>,
        Changed<Metallic>,
        Changed<Reflectance>,
    )>,
    Or<(
        Changed<Translucency>,
        Changed<SpecularStrength>,
        Changed<SpecularPower>,
        Changed<DiffuseScaling>,
        Changed<LightIntensity>,
        Changed<AmbientLightIntensity>,
    )>,
)>;

impl From<MaterialOptionDataItem<'_, '_>> for ScatterMaterialOptions {
    fn from(data: MaterialOptionDataItem<'_, '_>) -> Self {
        Self {
            general: GeneralOptions::from_data(&data),
            geometry: GeometryOptions::from_data(&data),
            wind: WindOptions::from_data(&data),
            bend: StaticBendOptions::from_data(&data),
            color: ColorOptions::from_data(&data),
            lighting: LightingOptions::from_data(&data),
            render_layers: data.render_layers.cloned(),
        }
    }
}

impl ScatterMaterialOptions {
    pub fn with(self, data: MaterialOptionDataItem) -> Self {
        Self {
            general: self.general.with_data(&data),
            geometry: self.geometry.with_data(&data),
            wind: self.wind.with_data(&data),
            bend: self.bend.with_data(&data),
            color: self.color.with_data(&data),
            lighting: self.lighting.with_data(&data),
            render_layers: data.render_layers.cloned().or(self.render_layers),
        }
    }

    pub fn with_options(mut self, other: Self) -> Self {
        self.general = self.general.with(other.general);
        self.geometry = self.geometry.with(other.geometry);
        self.wind = self.wind.with(other.wind);
        self.bend = self.bend.with(other.bend);
        self.color = self.color.with(other.color);
        self.lighting = self.lighting.with(other.lighting);
        self.render_layers = if other
            .render_layers
            .as_ref()
            .is_some_and(|x| *x != RenderLayers::default())
        {
            other.render_layers.clone()
        } else {
            self.render_layers
        };
        self
    }

    pub fn with_debug_color(mut self, debug_color: Color) -> Self {
        self.general.debug_color = debug_color;
        self
    }

    pub fn with_controlled(mut self, controlled: bool) -> Self {
        self.general.controlled = controlled;
        self
    }
}

pub trait ProtoType<T>
where
    T: Asset + Clone,
{
    fn mesh(&self) -> &Handle<Mesh>;
    fn material(&self) -> &Handle<T>;
    fn wind(&self) -> &Wind;
    fn aabb(&self) -> &Aabb;
    fn lod(&self) -> &LevelOfDetail;
    fn material_options(&self) -> &ScatterMaterialOptions;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}
