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
