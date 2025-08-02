use crate::core::Sampler;
use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use rand::Rng;

pub fn generate_scatter_points_layer_chunked(
    mut trigger: On<Scatter<ScatterLayer>>,
    mut cmd: Commands,
    q_root: Query<(
        Entity,
        &ChunkRoot,
        &BaseChunkSize,
        Option<&MapHeight>,
        &Aabb,
    )>,
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
        ),
        With<ScatterLayer>,
    >,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    chunk_query: Query<(&ChunkSize, &GlobalTransform), With<Chunk>>,
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
    )) = layer_query.get(trigger.target())
    else {
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok((root, child_chunks, base_chunk_size, map_height, aabb)) = q_root.get(**scatter_root)
    else {
        warn!("ScatterRoot not found!");
        return;
    };

    let density_sampler = get_density_sampler(pattern_dist, &images, *aabb);

    if !scatter_layer_enabled(&mut cmd, layer_entity, layer_name, enabled) {
        return;
    };

    let instances_dim = density_dist.map_or(10., |d| **d);

    for chunk_entity in child_chunks.iter() {
        info!(
            "Scattering {} instances in Chunk {}",
            (instances_dim as u32).pow(2),
            chunk_entity
        );
        let Ok((chunk_size, chunk_gtf)) = chunk_query.get(chunk_entity) else {
            warn!("Chunk not found!");
            continue;
        };

        let size = **base_chunk_size * Vec3::splat(**chunk_size as f32);

        let chunk_corner = chunk_gtf.translation() - size / 2.;

        let results = (0..(instances_dim as u32).pow(2))
            .filter_map(|i| {
                let x = i as f32 % instances_dim;
                let z = i as f32 / instances_dim;

                let mut instance_world_pos = chunk_corner
                    + Vec3::new(x * size.x / instances_dim, 0.0, z * size.z / instances_dim)
                    + get_jitter(jitter, &mut rng);

                instance_world_pos.y = match map_height {
                    None => 0.0,
                    Some(_) => height_sampler.sample(instance_world_pos),
                };

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

        let results = ScatterResults {
            results: results.clone(),
            chunk: Some(chunk_entity),
        };

        cmd.trigger_targets(results.clone(), [root, layer_entity, chunk_entity]);
        ew_results.write(results);
    }
}
