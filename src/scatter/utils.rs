use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use rand::Rng;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::collections::HashMap;
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
                .and_then(|x| create_height_map_sampler(images.get(&height_map_image.0), x))
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

// TODO refactor into async CPU/GPU pipelines
pub(super) fn create_scatter_result<R: Rng + ?Sized>(
    container: &Container,
    modifiers: &InstanceModifiers,
    rng: &mut R,
) -> Option<ScatterResult> {
    let instances_dim_f = container.instances_dim;
    let cell_width = container.size.x / instances_dim_f;
    let cell_depth = container.size.z / instances_dim_f;

    let world_corner_pos = container.transform.translation + container.corner;

    let local_cell_x_idx = rng.random_range(0.0..instances_dim_f).floor();
    let local_cell_z_idx = rng.random_range(0.0..instances_dim_f).floor();

    let snapped_world_cell_corner = world_corner_pos
        + Vec3::new(
            local_cell_x_idx * cell_width,
            0.0,
            local_cell_z_idx * cell_depth,
        );

    let cell_center_world_pos =
        snapped_world_cell_corner + Vec3::new(cell_width / 2.0, 0.0, cell_depth / 2.0);

    let jitter_strength = modifiers.jitter.map_or(1.0, |j| **j).clamp(0.0, 1.0);

    let max_offset_x = (cell_width * jitter_strength) / 2.0;
    let max_offset_z = (cell_depth * jitter_strength) / 2.0;

    let random_offset = Vec3::new(
        rng.random_range(-max_offset_x..max_offset_x),
        0.0,
        rng.random_range(-max_offset_z..max_offset_z),
    );

    let final_world_pos = cell_center_world_pos + random_offset;

    let mut instance_pos = final_world_pos - container.transform.translation;

    instance_pos.y = match modifiers.map_height {
        None => container.height,
        Some(_) => {
            modifiers.height_sampler.sample(final_world_pos) - container.transform.translation.y
        }
    };

    if let Some(sampler) = &modifiers.density_sampler {
        if rng.random::<f32>() > sampler.sample(final_world_pos) {
            return None;
        }
    }

    let final_scale = modifiers
        .scale
        .map_or(1.0, |s| rng.random_range(s.min..s.max));

    let final_rotation = modifiers.rotation.map_or(Quat::IDENTITY, |r| {
        Quat::from_rotation_y(rng.random_range(r.min..r.max))
    });

    Some(ScatterResult(Transform {
        translation: instance_pos,
        rotation: final_rotation,
        scale: Vec3::splat(final_scale),
    }))
}

pub(super) fn create_scatter_results<TIn, TOut>(
    container: Container,
    modifiers: InstanceModifiers,
) -> ScatterResults<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let mut rng = Pcg64::seed_from_u64(container.seed);

    let data = (0..(container.instances_dim as u32).pow(2))
        .filter_map(|_| create_scatter_result(&container, &modifiers, &mut rng))
        .collect::<Vec<_>>();

    ScatterResults::<TIn, TOut>::from(&container).with_data(data)
}

/// Generates a deterministic u64 seed by combining the global `WorldSeed` with location-specific data (like chunk coordinates
/// or an entity's position).
pub fn generate_seed(world_seed: &WorldSeed, location_data: impl Hash) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    location_data.hash(&mut hasher);
    let location_bytes = hasher.finish().to_le_bytes();

    hash64_with_seed(&location_bytes, world_seed.get())
}

/// Selects a prototype deterministically based on a seed, then finds the correct LOD.
pub fn select_consistent_prototype<'a, M: Asset + Clone>(
    items: &Vec<ScatterItemAsset<M>>,
    seed: u64,
    prototype_assets: &'a Assets<ScatterAsset<M>>,
    chunk_level: &ChunkLevel,
    is_chunk_spawn: bool,
) -> Option<(Name, &'a ScatterAsset<M>)> {
    let prototypes: Vec<&ScatterAsset<M>> = items
        .iter()
        .filter_map(|handle| prototype_assets.get(&**handle))
        .collect();

    if prototypes.is_empty() {
        warn!("No prototype assets could be loaded from handles.");
        return None;
    }

    let mut name_map: HashMap<Name, Vec<&ScatterAsset<M>>> = HashMap::new();
    prototypes.iter().for_each(|prototype| {
        let name = prototype.name.clone().unwrap_or_else(|| Name::new(""));
        name_map.entry(name).or_default().push(prototype);
    });

    if name_map.is_empty() {
        warn!("No prototypes to spawn after grouping.");
        return None;
    }

    let mut sorted_names: Vec<&Name> = name_map.keys().collect();
    sorted_names.sort();

    let mut rng = Pcg64::seed_from_u64(seed);
    let chosen_name = sorted_names.choose(&mut rng)?;
    let prototype_group = name_map.get(*chosen_name)?;

    let prototype = prototype_group
        .iter()
        .find(|p| !is_chunk_spawn || *p.lod_level == **chunk_level)
        .map(|p| *p)?;

    Some(((*chosen_name).clone(), prototype))
}
