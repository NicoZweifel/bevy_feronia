use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use rand::Rng;
use rand::prelude::*;

pub fn get_jitter(jitter: Option<&InstanceJitter>, rng: &mut ThreadRng) -> Vec3 {
    jitter.map_or_else(
        || Vec3::ZERO,
        |x| Vec3::new(rng.random_range(-**x..**x), 0., rng.random_range(-**x..**x)),
    )
}

pub fn get_height_map_sampler<'a>(
    images: &'a Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
) -> HeightMapSampler<'a> {
    let height_map_image = match height_map {
        None => None,
        Some(x) => images.get(&x.0),
    };

    height_map_cfg.map_or_else(
        || HeightMapSampler::Default(DefaultSampler),
        |cfg| {
            height_map_image.map_or_else(
                || HeightMapSampler::Default(DefaultSampler),
                |img| HeightMapSampler::CpuHeightMap(HeightMapCpuSampler::new(img, cfg.world_size)),
            )
        },
    )
}

pub fn scatter_layer_enabled<'a>(
    cmd: &'a mut Commands,
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

pub fn get_density_sampler<'a>(
    pattern_dist: Option<&DistributionPattern>,
    images: &'a Res<Assets<Image>>,
    aabb: Aabb,
) -> Option<DensityMapSampler<'a>> {
    pattern_dist
        .and_then(|p| images.get(&p.density_map))
        .map(|density_image| {
            DensityMapSampler::new(density_image, Vec3::from(aabb.half_extents * 2.))
        })
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

pub(super) fn create_scatter_result(
    i: u32,
    container: &Container,
    modifiers: &InstanceModifiers,
    rng: &mut ThreadRng,
) -> Option<ScatterResult> {
    let x = i as f32 % container.instances_dim;
    let z = i as f32 / container.instances_dim;

    let jitter_value = modifiers.jitter.map_or(0., |x| **x);

    let mut instance_world_pos = container.corner
        + Vec3::new(
            x * container.size.x / (container.instances_dim - jitter_value * 2.),
            0.0,
            z * container.size.z / container.instances_dim,
        )
        + get_jitter(modifiers.jitter, rng);

    instance_world_pos.y = match modifiers.map_height {
        None => container.height,
        Some(_) => modifiers.height_sampler.sample(instance_world_pos),
    };

    if let Some(sampler) = &modifiers.density_sampler {
        if rng.random::<f32>() > sampler.sample(instance_world_pos) {
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
        translation: instance_world_pos,
        rotation: final_rotation,
        scale: Vec3::splat(final_scale),
    }))
}

pub(super) struct InstanceModifiers<'a> {
    pub map_height: Option<&'a MapHeight>,
    pub height_sampler: &'a HeightMapSampler<'a>,
    pub density_sampler: &'a Option<DensityMapSampler<'a>>,
    pub scale: Option<&'a InstanceScale>,
    pub rotation: Option<&'a InstanceRotationYaw>,
    pub jitter: Option<&'a InstanceJitter>,
}

pub(super) struct Container {
    pub layer_entity: Entity,
    pub chunk_entity: Option<Entity>,
    pub instances_dim: f32,
    pub corner: Vec3,
    pub height: f32,
    pub size: Vec3,
}

pub(super) fn create_scatter_results(
    container: Container,
    modifiers: InstanceModifiers,
) -> ScatterResults {
    let mut rng = rand::rng();
    ScatterResults {
        data: (0..(container.instances_dim as u32).pow(2))
            .filter_map(|i| create_scatter_result(i, &container, &modifiers, &mut rng))
            .collect::<Vec<_>>(),
        layer: container.layer_entity,
        chunk: container.chunk_entity,
    }
}
