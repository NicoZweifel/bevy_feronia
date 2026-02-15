use crate::prelude::*;

use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_math::Vec3;
use bevy_transform::prelude::GlobalTransform;

use std::hash::{Hash, Hasher};
use xxh3::hash64_with_seed;

#[cfg(feature = "trace")]
use tracing::warn;

pub fn scatter_layer_enabled(
    layer_entity: Entity,
    layer_name: Option<&Name>,
    enabled: Option<&ScatterLayerEnabled>,
) -> bool {
    if !*enabled.cloned().unwrap_or(true.into()) {
        let _name = layer_name
            .unwrap_or(&Name::new(layer_entity.to_string()))
            .to_string();

        #[cfg(feature = "trace")]
        debug!("ScatterLayer {_name} is disabled!");
        return false;
    }

    true
}

pub fn combine_aabbs(aabb1: &Aabb, aabb2: &Aabb) -> Aabb {
    let min_x = aabb1.min().x.min(aabb2.min().x);
    let min_y = aabb1.min().y.min(aabb2.min().y);
    let min_z = aabb1.min().z.min(aabb2.min().z);
    let max_x = aabb1.max().x.max(aabb2.max().x);
    let max_y = aabb1.max().y.max(aabb2.max().y);
    let max_z = aabb1.max().z.max(aabb2.max().z);

    Aabb::from_min_max(
        Vec3::new(min_x, min_y, min_z),
        Vec3::new(max_x, max_y, max_z),
    )
}

/// Used in CPU Scatter Tasks to avoid copying/cloning data for every single instance.
#[derive(Copy, Clone)]
pub struct InstanceModifiers<'a> {
    pub map_height: Option<&'a MapHeight>,
    pub height_sampler: &'a HeightMapSampler<'a>,
    pub density_sampler: Option<&'a DensityMapSampler<'a>>,
    pub scale: Option<&'a InstanceScaleRange>,
    pub rotation: Option<&'a InstanceRotationYawRange>,
    pub jitter: Option<&'a InstanceJitterStrength>,
    pub avoidance: Option<&'a Avoidance>,
    pub density: Option<&'a LodDensity>,
}

impl<'a> From<&'a ScatterTaskData> for InstanceModifiers<'a> {
    fn from(value: &'a ScatterTaskData) -> Self {
        Self {
            jitter: value.jitter.as_ref(),
            avoidance: value.avoidance.as_ref(),
            map_height: value.map_height.as_ref(),
            scale: value.scale.as_ref(),
            rotation: value.rotation.as_ref(),
            density: value.density.as_ref(),
            height_sampler: &HeightMapSampler::Default(DefaultSampler),
            density_sampler: None,
        }
    }
}

impl<'a> InstanceModifiers<'a> {
    pub fn with_height_sampler(mut self, height_sampler: &'a HeightMapSampler<'a>) -> Self {
        self.height_sampler = height_sampler;
        self
    }

    pub fn with_density_sampler(
        mut self,
        density_sampler: Option<&'a DensityMapSampler<'a>>,
    ) -> Self {
        self.density_sampler = density_sampler;
        self
    }
}

#[derive(Clone)]
pub struct Container {
    pub entity: Entity,
    pub layer_entity: Entity,
    pub chunk_entity: Option<Entity>,
    pub root_entity: Entity,
    pub instances_dim: f32,
    pub corner: Vec3,
    pub height: f32,
    pub size: Vec3,
    pub root_size: Vec3,
    pub global_transform: GlobalTransform,
    pub root_global_transform: GlobalTransform,
    pub seed: u64,
}

/// Generates a deterministic u64 seed by combining the global `WorldSeed` with location-specific data (like chunk coordinates
/// or an entity's position).
pub fn generate_seed(world_seed: &WorldSeed, location_data: impl Hash) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    location_data.hash(&mut hasher);
    let location_bytes = hasher.finish().to_le_bytes();

    hash64_with_seed(&location_bytes, world_seed.get())
}

pub fn generate_instance_seed(base_seed: u64, world_position: Vec3) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    base_seed.hash(&mut hasher);

    world_position.as_ivec3().hash(&mut hasher);

    hasher.finish()
}
