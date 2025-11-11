use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilityRange;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;

pub trait ScatterMaterial<TIn = StandardMaterial>: Asset + Clone
where
    TIn: Material,
{
    fn create_material(
        base: Option<TIn>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> Self;

    fn update_material(_material: &mut Self, _wind: Wind, _options: MaterialOptions) {}

    fn component(material: Handle<Self>) -> impl Component;

    fn spawn(cmd: &mut Commands, request: SpawnRequest<Self>);
}

pub struct SpawnRequest<'w, T>
where
    T: Asset + Clone,
{
    pub event: &'w SpawnScatterAssets<T>,
    pub names: Vec<Name>,
    pub name_map: &'w HashMap<Name, Vec<ScatterHandleAsset<'w, T>>>,
    pub is_chunked: bool,
    pub chunk_level: ChunkLevel,
    pub chunk_gtf_translation: Vec3,
    pub lod_config: &'w LodConfig,
    pub parent: Entity,
}

pub struct ScatterHandleAsset<'w, T>
where
    T: Asset + Clone,
{
    pub handle: Handle<ScatterAsset<T>>,
    pub asset: &'w ScatterAsset<T>,
}

impl<T> ScatterHandleAsset<'_, T>
where
    T: Asset + Clone,
{
    pub fn is_lod(&self, chunked: bool, lod: u32) -> bool {
        if chunked {
            *self.asset.properties.lod == lod
        } else {
            *self.asset.properties.lod >= lod
        }
    }
}

impl<'w, T> SpawnRequest<'w, T>
where
    T: Asset + Clone,
{
    pub fn prototypes_from_seed(
        &'w self,
        seed: u64,
    ) -> impl Iterator<Item = &'w ScatterHandleAsset<'w, T>> {
        let mut rng = Pcg64::seed_from_u64(seed);

        self.names
            .choose(&mut rng)
            .into_iter()
            .flat_map(move |name| self.prototypes_from_name(name))
    }

    pub fn prototypes_from_name(
        &'w self,
        name: &Name,
    ) -> impl Iterator<Item = &'w ScatterHandleAsset<'w, T>> {
        self.name_map
            .get(name)
            .map_or(&[][..], |prototypes| prototypes.as_slice())
            .iter()
            .filter(move |&handle_asset| handle_asset.is_lod(self.is_chunked, *self.chunk_level))
    }
}

impl<'w, T> SpawnRequest<'w, T>
where
    T: Material,
{
    pub fn batch_spawn_material(
        self,
    ) -> Vec<(
        Transform,
        Mesh3d,
        MeshMaterial3d<T>,
        ChildOf,
        VisibilityRange,
        ScatteredInstance,
        ScatteredAsset<T>,
    )> {
        self.event
            .trigger
            .data
            .iter()
            .flat_map(|res| {
                self.prototypes_from_seed(res.seed)
                    .map(|ScatterHandleAsset { handle, asset }| {
                        let visibility_range =
                            self.lod_config.get_visibility_range(asset.properties.lod);
                        (
                            res.transform,
                            Mesh3d(asset.mesh().clone()),
                            MeshMaterial3d::<T>(asset.material().clone()),
                            ChildOf(self.parent),
                            visibility_range,
                            ScatteredInstance(self.event.trigger.layer),
                            ScatteredAsset(handle.clone()),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

impl ScatterMaterial for StandardMaterial {
    fn create_material(
        base: Option<StandardMaterial>,
        _noise_texture: Handle<Image>,
        _properties: &ScatterAssetProperties,
    ) -> StandardMaterial {
        base.unwrap_or_default()
    }

    fn component(material: Handle<StandardMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<StandardMaterial>) {
        cmd.spawn_batch(request.batch_spawn_material());
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Zeroable, Copy)]
pub struct WindUniform {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub bend_exponent: f32,
    pub micro_strength: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,

    // TODO move to uniform for Options/InstanceData
    pub edge_correction_factor: f32,
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
    pub debug_color: Vec4,
}

impl From<&Wind> for WindUniform {
    fn from(wind: &Wind) -> Self {
        WindUniform {
            direction: wind.direction,
            strength: wind.strength,
            noise_scale: wind.noise_scale,
            scroll_speed: wind.scroll_speed,
            bend_exponent: wind.bend_exponent,
            micro_strength: wind.micro_strength,
            s_curve_speed: wind.s_curve_speed,
            s_curve_strength: wind.s_curve_strength,
            s_curve_frequency: wind.s_curve_frequency,
            bop_speed: wind.bop_speed,
            bop_strength: wind.bop_strength,
            twist_strength: wind.twist_strength,
            edge_correction_factor: 0.,
            aabb_max: Vec3::splat(1.),
            aabb_min: Vec3::splat(0.),
            debug_color: Vec4::splat(1.),
        }
    }
}

impl WindUniform {
    pub fn with_edge_correction_factor(mut self, edge_correction_factor: f32) -> Self {
        self.edge_correction_factor = edge_correction_factor;
        self
    }

    pub fn with_aabb(mut self, aabb: &Aabb) -> Self {
        self.aabb_min = aabb.min().into();
        self.aabb_max = aabb.max().into();
        self
    }

    pub fn with_debug_color(mut self, color: Vec4) -> Self {
        self.debug_color = color;
        self
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
    pub struct WindAffectedKey: u64 {
        const BILLBOARDING    = 1 << 0;
        const EDGE_CORRECTION = 1 << 1;
        const WIND_LOW_QUALITY = 1 << 2;
        const FAST_NORMALS = 1 << 3;
        const DEBUG = 1 << 4;
        const WIND_AFFECTED= 1 << 5;
        const SUBSURFACE_SCATTERING = 1 << 6;
        const STATIC_BEND = 1 << 7;
        const ANALYTICAL_NORMALS = 1 << 8;
        const CURVE_NORMALS = 1 << 9;
        const POINT_LIGHTS = 1 << 10;
    }
}
