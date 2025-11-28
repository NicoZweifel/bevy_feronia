use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;

use bevy_asset::{Asset, Handle};
use bevy_camera::{primitives::Aabb, visibility::VisibilityRange};
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{Material, MeshMaterial3d, StandardMaterial};
use bevy_platform::collections::HashMap;
use bevy_render::render_resource::ShaderType;
use bevy_transform::prelude::Transform;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::fmt::Debug;

#[cfg(feature = "avian")]
use avian3d::prelude::Collider;

pub trait ScatterMaterialAsset: Asset + Clone + Default + Debug {}

impl<T> ScatterMaterialAsset for T where T: Asset + Clone + Default + Debug {}

pub trait ScatterMaterial: ScatterMaterialAsset {
    fn create_material(
        base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> Self;

    fn update_material(_material: &mut Self, _wind: Wind, _options: ScatterMaterialOptions) {}

    fn component(material: Handle<Self>) -> impl Component;

    fn spawn(cmd: &mut Commands, request: SpawnRequest<Self>);
}

pub struct SpawnRequest<'w, T>
where
    T: ScatterMaterialAsset,
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
    T: ScatterMaterialAsset,
{
    pub handle: Handle<ScatterAsset<T>>,
    pub asset: &'w ScatterAsset<T>,
}

impl<T> ScatterHandleAsset<'_, T>
where
    T: ScatterMaterialAsset,
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
    T: ScatterMaterialAsset,
{
    pub fn prototypes_from_seed_iter(
        &self,
        seed: u64,
    ) -> impl Iterator<Item = &'w ScatterHandleAsset<'w, T>> {
        let mut rng = Pcg64::seed_from_u64(seed);

        self.names
            .choose(&mut rng)
            .into_iter()
            .flat_map(|name| self.prototypes_from_name_iter(name))
    }

    pub fn prototypes_from_name_iter(
        &self,
        name: &Name,
    ) -> impl Iterator<Item = &'w ScatterHandleAsset<'w, T>> {
        self.name_map
            .get(name)
            .map_or(&[][..], |prototypes| prototypes.as_slice())
            .iter()
            .filter(|&handle_asset| handle_asset.is_lod(self.is_chunked, *self.chunk_level))
    }
}

#[cfg(not(feature = "avian"))]
type SpawnRequestItem<T> = (
    Transform,
    VisibilityRange,
    Mesh3d,
    MeshMaterial3d<T>,
    ChildOf,
    ScatteredInstance,
    ScatteredAsset<T>,
);

#[cfg(feature = "avian")]
type SpawnRequestItem<T> = (
    Transform,
    VisibilityRange,
    Mesh3d,
    MeshMaterial3d<T>,
    ChildOf,
    ScatteredInstance,
    ScatteredAsset<T>,
    Collider,
);

impl<'w, T> SpawnRequest<'w, T>
where
    T: Material + Default + Debug,
{
    pub fn spawn_batch_iter(&self) -> impl Iterator<Item = SpawnRequestItem<T>> {
        self.event.trigger.data.iter().flat_map(|res| {
            self.prototypes_from_seed_iter(res.seed).flat_map(
                |ScatterHandleAsset { handle, asset }| {
                    asset.parts.iter().map(|part| {
                        (
                            res.transform.mul_transform(part.transform),
                            self.lod_config.get_visibility_range(asset.properties.lod),
                            Mesh3d(part.h_mesh.clone()),
                            MeshMaterial3d::<T>(part.h_material.clone()),
                            ChildOf(self.parent),
                            ScatteredInstance(self.event.trigger.layer),
                            ScatteredAsset(handle.clone()),
                            // TODO find a method for conditionally adding colliders
                            #[cfg(feature = "avian")]
                            part.collider.clone().unwrap_or_default(),
                        )
                    })
                },
            )
        })
    }
}

impl ScatterMaterial for StandardMaterial {
    fn create_material(
        base: Option<StandardMaterial>,
        _noise_texture: Handle<Image>,
        _properties: &ScatterAssetProperties,
    ) -> StandardMaterial {
        base.unwrap_or_default().into()
    }

    fn component(material: Handle<StandardMaterial>) -> impl Component {
        MeshMaterial3d(material)
    }

    fn spawn(cmd: &mut Commands, request: SpawnRequest<StandardMaterial>) {
        cmd.spawn_batch(request.spawn_batch_iter().collect::<Vec<_>>());
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Zeroable, Copy)]
pub struct WindUniform {
    pub direction: Vec2,
    pub strength: f32,
    pub noise_scale: f32,
    pub scroll_speed: f32,
    pub micro_strength: f32,
    pub s_curve_speed: f32,
    pub s_curve_strength: f32,
    pub s_curve_frequency: f32,
    pub bop_speed: f32,
    pub bop_strength: f32,
    pub twist_strength: f32,
    pub edge_correction_factor: f32,

    /// TODO use in both materials or rename [`WindUniform`] to `ExtendedUniforms`
    pub sss_strength: f32,
    pub sss_scale: f32,

    /// TODO use in both materials move to separate uniform e.g.
    /// or move to [`InstanceUniforms`]
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
            sss_strength: 0.,
            sss_scale: 0.,
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

    pub fn with_sss(mut self, sss_strength: f32, sss_scale: f32) -> Self {
        self.sss_strength = sss_strength;
        self.sss_scale = sss_scale;
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
        const STATIC_SHADOW = 1<< 6;
        const STATIC_BEND = 1 << 7;
        const ANALYTICAL_NORMALS = 1 << 8;
        const CURVE_NORMALS = 1 << 9;
         /// TODO use in both materials create separate keys
        const SUBSURFACE_SCATTERING = 1 << 10;
        const POINT_LIGHTS = 1 << 11;
        const DIRECTIONAL_LIGHTS = 1 << 12;
        const GPU_CULL = 1 << 13;
    }
}
