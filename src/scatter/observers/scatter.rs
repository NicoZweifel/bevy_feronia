use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

type LayerQueryType<'a> = (
    Entity,
    &'a ScatterLayerOf,
    Option<&'a DistributionDensity>,
    Option<&'a DistributionPattern>,
    Option<&'a InstanceRotationYaw>,
    Option<&'a InstanceScale>,
    Option<&'a InstanceJitter>,
    &'a GlobalTransform,
);

pub fn scatter<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut trigger: On<Scatter<TIn, TOut>>,
    mut cmd: Commands,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    images: Res<Assets<Image>>,
    q_root: Query<(Entity, Option<&MapHeight>, &Aabb), (Without<ChunkRoot>, With<ScatterRoot>)>,
    q_layer: Query<LayerQueryType, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
    mut ew_results: EventWriter<ScatterResults<TIn, TOut>>,
) {
    trigger.propagate(false);

    let height_sampler = get_height_map_sampler(&images, height_map_cfg, height_map);

    let Ok((
        layer_entity,
        scatter_root,
        density_dist,
        pattern_dist,
        rotation,
        scale,
        jitter,
        layer_gtf,
    )) = q_layer.get(trigger.target())
    else {
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok((root, map_height, aabb)) = q_root.get(**scatter_root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    let density_sampler = get_density_sampler(pattern_dist, &images, *aabb);

    let instances_dim = density_dist.map_or(10., |d| **d);

    let size = Vec3::from(aabb.half_extents * 2.0);

    info!(
        "Scattering {} instances in ScatterLayer {}",
        (instances_dim as u32).pow(2),
        layer_entity,
    );

    let corner = -Vec3::from(aabb.half_extents);

    let results = create_scatter_results(
        Container {
            layer_entity,
            chunk_entity: None,
            instances_dim,
            corner,
            height: 0.,
            size,
            transform: layer_gtf.compute_transform(),
            root_entity: root,
        },
        InstanceModifiers {
            jitter,
            map_height,
            height_sampler: &height_sampler,
            density_sampler: &density_sampler,
            scale,
            rotation,
        },
    );

    info!("Scattered {} instances", results.data.len());

    cmd.trigger_targets(results.clone(), [root, layer_entity]);
    ew_results.write(results);
}
