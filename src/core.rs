use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Event, Message, Debug, Clone)]
pub struct SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub items: Vec<ScatterItemAsset<T>>,
    pub trigger: SpawnTrigger,
}

#[derive(Clone, Debug)]
pub struct SpawnTrigger {
    pub chunk: Option<Entity>,
    pub layer: Entity,
    pub root: Entity,
    pub target: Entity,
    pub data: Vec<ScatterResult>,
    pub seed: u64,
}

#[derive(Clone, Debug, Reflect, Copy)]
pub struct MaterialOptions {
    // Determines whether the material is controlled externally and
    // isn't automatically updated and kept in sync with the Wind resource.
    pub controlled: bool,

    pub debug_color: Color,

    pub debug: bool,

    pub enable_billboarding: bool,
    pub fast_normals: bool,
    pub edge_correction_factor: f32,
    pub curve_factor: f32,
    pub lod_threshold: f32,
    pub wind_affected: bool,
    pub low_quality: bool,
    pub color: Option<Color>,
}

impl Default for MaterialOptions {
    fn default() -> Self {
        Self {
            controlled: false,
            debug_color: Default::default(),
            debug: false,
            enable_billboarding: false,
            fast_normals: false,
            edge_correction_factor: 0.0,
            curve_factor: 0.0,
            // TODO sync/cleanup with LOD systems / chunks systems
            lod_threshold: 50.,
            wind_affected: false,
            low_quality: false,
            color: None,
        }
    }
}

pub type MaterialOptionData<'w> = (
    Option<&'w EnableDebug>,
    Option<&'w EnableBillboarding>,
    Option<&'w EdgeCorrectionFactor>,
    Option<&'w CurveFactor>,
    Option<&'w WindAffected>,
    Option<&'w LowQuality>,
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
        ): MaterialOptionData,
    ) -> Self {
        Self {
            debug: enable_debug.is_some(),
            enable_billboarding: enable_billboarding.is_some(),
            edge_correction_factor: edge_correction_factor.map(|x| **x).unwrap_or(0.),
            curve_factor: curve_factor.map(|x| **x).unwrap_or(0.),
            wind_affected: wind_affected.is_some(),
            low_quality: low_q.is_some(),
            ..default()
        }
    }
}

impl MaterialOptions {
    pub fn with(
        &self,
        (
            enable_debug,
            enable_billboarding,
            edge_correction_factor,
            curve_factor,
            wind_affected,
            low_q,
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
            ..*self
        }
    }

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

    pub fn with_quality(mut self, lod_level: u32, wind_affected: bool) -> Self {
        self.low_quality = lod_level > 1;
        self.wind_affected = lod_level < 3 && wind_affected;
        self
    }
}

impl<T> SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    pub fn new(items: Vec<ScatterItemAsset<T>>, trigger: SpawnTrigger) -> Self {
        Self { items, trigger }
    }

    pub fn with_items(mut self, items: Vec<ScatterItemAsset<T>>) -> Self {
        self.items = items;
        self
    }
}

impl<T> From<SpawnTrigger> for SpawnProtoTypes<T>
where
    T: Asset + Clone,
{
    fn from(value: SpawnTrigger) -> Self {
        Self::new(Vec::new(), value)
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
    fn material_options(&self) -> &MaterialOptions;
}

pub trait Sampler {
    fn sample(&self, world_pos: Vec3) -> f32;
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LevelOfDetail(pub u32);

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableDebug;

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EnableBillboarding;

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct EdgeCorrectionFactor(pub f32);

impl Default for EdgeCorrectionFactor {
    fn default() -> Self {
        Self(0.01)
    }
}

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct CurveFactor(pub f32);

impl Default for CurveFactor {
    fn default() -> Self {
        Self(TAU)
    }
}

#[derive(Clone, Default)]
pub struct ThreadSafeImage {
    /// Raw pixel data.
    pub pixels: Vec<u8>,
    pub dimensions: UVec2,
}
