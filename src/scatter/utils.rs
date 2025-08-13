use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use rand::Rng;
use rand::prelude::*;

pub fn get_height_map_sampler<'a>(
    images: &'a Res<Assets<Image>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
) -> HeightMapSampler<'a> {
    height_map
        .map(|height_map_image| {
            height_map_cfg
                .map(|x| create_height_map_sampler(images.get(&height_map_image.0), x))
                .flatten()
        })
        .flatten()
        .unwrap_or_else(|| HeightMapSampler::Default(DefaultSampler))
}

fn create_height_map_sampler<'a>(
    height_map_image: Option<&'a Image>,
    cfg: Res<HeightMapConfig>,
) -> Option<HeightMapSampler<'a>> {
    height_map_image
        .map(|img| HeightMapSampler::Cpu(HeightMapCpuSampler::new(img, cfg.into_inner())))
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
    pub root_entity: Entity,
    pub instances_dim: f32,
    pub corner: Vec3,
    pub height: f32,
    pub size: Vec3,
    pub transform: Transform,
}

// TODO refactor into async CPU/GPU pipelines
pub(super) fn create_scatter_result(
    container: &Container,
    modifiers: &InstanceModifiers,
    rng: &mut ThreadRng,
) -> Option<ScatterResult> {
    let instances_dim_f = container.instances_dim;
    let cell_width = container.size.x / instances_dim_f;
    let cell_depth = container.size.z / instances_dim_f;

    let world_corner_pos = container.transform.translation + container.corner;
    let start_grid_x = (world_corner_pos.x / cell_width).round();
    let start_grid_z = (world_corner_pos.z / cell_depth).round();

    let local_cell_x_idx = rng.random_range(0.0..instances_dim_f).floor();
    let local_cell_z_idx = rng.random_range(0.0..instances_dim_f).floor();

    let snapped_world_cell_corner = Vec3::new(
        (start_grid_x + local_cell_x_idx) * cell_width,
        0.0,
        (start_grid_z + local_cell_z_idx) * cell_depth,
    );

    let jitter_strength = modifiers.jitter.map_or(1.0, |j| **j);
    let margin_x = (cell_width * (1.0 - jitter_strength)) / 2.0;
    let margin_z = (cell_depth * (1.0 - jitter_strength)) / 2.0;

    let final_world_pos = snapped_world_cell_corner
        + Vec3::new(
            margin_x + (rng.random::<f32>() * cell_width * jitter_strength),
            0.0,
            margin_z + (rng.random::<f32>() * cell_depth * jitter_strength),
        );

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
    let mut rng = rand::rng();

    let data = (0..(container.instances_dim as u32).pow(2))
        .filter_map(|_| create_scatter_result(&container, &modifiers, &mut rng))
        .collect::<Vec<_>>();

    ScatterResults::<TIn, TOut>::from(&container).with_data(data)
}
