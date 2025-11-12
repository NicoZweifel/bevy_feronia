use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use std::hash::Hash;
use std::hash::Hasher;
use xxh3::hash64_with_seed;

pub fn get_height_map_sampler<'a>(
    images: &'a Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
) -> HeightMapSampler<'a> {
    height_map
        .and_then(|height_map_image| {
            height_map_cfg
                .and_then(|cfg| create_height_map_sampler(images.get(&height_map_image.0), cfg))
        })
        .unwrap_or(HeightMapSampler::Default(DefaultSampler))
}

fn create_height_map_sampler<'a>(
    height_map_image: Option<&'a Image>,
    cfg: Res<HeightMapConfig>,
) -> Option<HeightMapSampler<'a>> {
    height_map_image
        .map(|img| HeightMapSampler::Cpu(HeightMapCpuSampler::new(img, cfg.into_inner())))
}

pub fn scatter_layer_enabled(
    cmd: &mut Commands,
    layer_entity: Entity,
    layer_name: Option<&Name>,
    enabled: Option<&ScatterLayerEnabled>,
) -> bool {
    let scatter_layer_enabled = ScatterLayerEnabled(true);

    if !**enabled.unwrap_or(&scatter_layer_enabled) {
        let name = layer_name
            .unwrap_or(&Name::new(layer_entity.to_string()))
            .to_string();

        warn!("ScatterLayer {name} is disabled!");
        return false;
    }

    cmd.entity(layer_entity).insert(scatter_layer_enabled);

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

pub struct InstanceModifiers<'a> {
    pub map_height: Option<&'a MapHeight>,
    pub height_sampler: &'a HeightMapSampler<'a>,
    pub density_sampler: &'a Option<DensityMapSampler<'a>>,
    pub scale: Option<&'a InstanceScale>,
    pub rotation: Option<&'a InstanceRotationYaw>,
    pub jitter: Option<&'a InstanceJitter>,
    pub avoidance: Option<&'a Avoidance>,
    pub density: Option<&'a LodDensity>,
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
    pub transform: Transform,
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
