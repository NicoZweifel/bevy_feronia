use crate::core::Sampler;
use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use rand::Rng;

pub fn generate_scatter_points_layer(
    mut trigger: On<Scatter<ScatterLayer>>,
    mut cmd: Commands,
    q_root: Query<(Entity, Option<&MapHeight>, &Aabb), (Without<ChunkRoot>, With<ScatterRoot>)>,
    layer_query: Query<
        (
            Entity,
            &ScatterLayerOf,
            Option<&Name>,
            Option<&DistributionDensity>,
            Option<&DistributionPattern>,
            Option<&InstanceRotationYaw>,
            Option<&InstanceScale>,
            Option<&ScatterLayerEnabled>,
            Option<&InstanceJitter>,
            &GlobalTransform,
        ),
        With<ScatterLayer>,
    >,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    images: Res<Assets<Image>>,
    mut ew_results: EventWriter<ScatterResults>,
) {
    trigger.propagate(false);

    let height_sampler = get_height_map_sampler(&images, height_map_cfg, height_map);

    let mut rng = rand::rng();

    let Ok((
        layer_entity,
        scatter_root,
        layer_name,
        density_dist,
        pattern_dist,
        rotation,
        scale,
        enabled,
        jitter,
        layer_gtf,
    )) = layer_query.get(trigger.target())
    else {
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok((root, map_height, aabb)) = q_root.get(**scatter_root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    let density_sampler = get_density_sampler(pattern_dist, &images, *aabb);

    if !scatter_layer_enabled(&mut cmd, layer_entity, layer_name, enabled) {
        return;
    };

    let instances_dim = density_dist.map_or(10., |d| **d);

    let size = aabb.half_extents * 2.0;

    info!(
        "Scattering {} instances in ScatterLayer {}",
        (instances_dim as u32).pow(2),
        layer_entity
    );

    let jitter_value = jitter.map_or(0., |x| **x);

    let corner =
        layer_gtf.translation() - Vec3::from(aabb.half_extents) + Vec3::splat(jitter_value);

    let results = (0..(instances_dim as u32).pow(2))
        .filter_map(|i| {
            let x = i as f32 % instances_dim;
            let z = i as f32 / instances_dim;

            let mut instance_world_pos = corner
                + Vec3::new(
                    x * (size.x - jitter_value * 2.) / instances_dim,
                    0.0,
                    z * (size.z - jitter_value * 2.) / instances_dim,
                )
                + get_jitter(jitter, &mut rng);

            instance_world_pos.y = map_height.map_or_else(
                || layer_gtf.translation().y,
                |_| height_sampler.sample(instance_world_pos),
            );

            if let Some(sampler) = &density_sampler {
                if rng.random::<f32>() > sampler.sample(instance_world_pos) {
                    return None;
                }
            }

            let final_scale = scale.map_or(1.0, |s| rng.random_range(s.min..s.max));

            let final_rotation = rotation.map_or(Quat::IDENTITY, |r| {
                Quat::from_rotation_y(rng.random_range(r.min..r.max))
            });

            Some(ScatterResult {
                layer: layer_entity,
                global_transform: Transform {
                    translation: instance_world_pos,
                    rotation: final_rotation,
                    scale: Vec3::splat(final_scale),
                },
            })
        })
        .collect::<Vec<_>>();

    info!("Scattered {} instances", results.len());

    let results = ScatterResults {
        results: results.clone(),
        chunk: None,
    };

    cmd.trigger_targets(results.clone(), [root, layer_entity]);
    ew_results.write(results);
}
