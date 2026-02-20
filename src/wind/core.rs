use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;

use bevy_asset::{Asset, Handle};
use bevy_camera::prelude::Visibility;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::*;
use bevy_mesh::Mesh3d;
use bevy_pbr::{Material, MeshMaterial3d, StandardMaterial};
use bevy_platform::collections::HashMap;
use bevy_render::render_resource::ShaderType;
use bevy_transform::components::GlobalTransform;
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::fmt::Debug;

pub trait ScatterMaterialAsset: Asset + Clone + Default + Debug {}

impl<T> ScatterMaterialAsset for T where T: Asset + Clone + Default + Debug {}

pub trait ScatterMaterial: ScatterMaterialAsset {
    fn create_material(
        base: Option<StandardMaterial>,
        noise_texture: Handle<Image>,
        properties: &ScatterAssetProperties,
    ) -> Self;

    fn update_material(
        _material: &mut Self,
        _current_wind: Wind,
        _previous_wind: Wind,
        _options: ScatterMaterialOptions,
    ) {
    }

    fn component(material: Handle<Self>) -> impl Component;

    fn spawn(cmd: &mut Commands, request: SpawnRequest<Self>);
}

pub struct SpawnRequest<'w, T>
where
    T: ScatterMaterialAsset,
{
    pub event: &'w SpawnScatterAssets<T>,
    pub name_map: &'w HashMap<Name, Vec<ScatterHandleAsset<'w, T>>>,
    pub is_chunked: bool,
    pub chunk_level: ChunkLevel,
    pub container_gtf: GlobalTransform,
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

        self.get_sorted_names()
            .choose(&mut rng)
            .copied()
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

    pub fn get_sorted_names(&self) -> Vec<&'w Name> {
        let mut names: Vec<&Name> = self.name_map.keys().collect();
        names.sort();
        names
    }
}

impl<'w, T> SpawnRequest<'w, T>
where
    T: Material + Default + Debug,
{
    pub(crate) fn spawn(&'w self, cmd: &mut Commands) {
        for (res, ScatterHandleAsset { handle, asset }) in
            self.event.trigger.data.iter().flat_map(move |res| {
                self.prototypes_from_seed_iter(res.seed)
                    .map(move |handle_asset| (res, handle_asset))
            })
        {
            let entity = cmd
                .spawn((
                    ChildOf(self.parent),
                    ScatteredInstance(self.event.trigger.layer),
                    ScatteredAsset(handle.clone()),
                    Visibility::Visible,
                    res.transform,
                ))
                .id();

            #[cfg(feature = "avian")]
            if let Some(body) = asset.rigid_body {
                cmd.entity(entity).insert(body);
            }

            cmd.spawn_batch(
                asset
                    .parts
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, part)| {
                        (
                            part.transform,
                            Visibility::Visible,
                            self.lod_config.get_visibility_range(asset.properties.lod),
                            Mesh3d(part.mesh().clone()),
                            MeshMaterial3d::<T>(part.material().clone()),
                            ChildOf(entity),
                            ScatteredPart((handle.clone(), i)),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
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
        request.spawn(cmd);
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Zeroable, Copy, Debug)]
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
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
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
            aabb_max: Vec3::splat(1.),
            aabb_min: Vec3::splat(0.),
        }
    }
}

impl WindUniform {
    pub fn with_aabb(mut self, aabb: &Aabb) -> Self {
        self.aabb_min = aabb.min().into();
        self.aabb_max = aabb.max().into();
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
        const WIND_AFFECTED= 1 << 4;
        const STATIC_BEND = 1 << 5;
        const ANALYTICAL_NORMALS = 1 << 6;
    }
}

impl From<&ScatterMaterialOptions> for WindAffectedKey {
    fn from(options: &ScatterMaterialOptions) -> Self {
        let mut key = WindAffectedKey::empty();

        let NormalOptions {
            analytical_normals,
            fast_normals,
            ..
        } = options.lighting.normals;
        let WindOptions {
            affected,
            low_quality,
        } = options.wind;
        let GeometryOptions {
            enable_billboarding,
            edge_correction_factor,
            ..
        } = options.geometry;
        let StaticBendOptions {
            strength: static_bend_strength,
            ..
        } = options.bend;

        key.set(WindAffectedKey::BILLBOARDING, enable_billboarding);
        key.set(
            WindAffectedKey::EDGE_CORRECTION,
            edge_correction_factor > 0.,
        );
        key.set(WindAffectedKey::WIND_LOW_QUALITY, low_quality);
        key.set(WindAffectedKey::FAST_NORMALS, fast_normals);
        key.set(WindAffectedKey::WIND_AFFECTED, affected);
        key.set(WindAffectedKey::STATIC_BEND, static_bend_strength > 0.);
        key.set(WindAffectedKey::ANALYTICAL_NORMALS, analytical_normals);

        key
    }
}
