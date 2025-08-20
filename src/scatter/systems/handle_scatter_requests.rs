use crate::height_map::cpu_sampler::HeightMapCpuSampler;
use crate::prelude::*;
use crate::scatter::utils::{Container, InstanceModifiers, create_scatter_results, generate_seed};
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::tasks::futures_lite::future;

type ScatterLayerQueryData<'a> = (
    &'a ScatterLayerOf,
    Option<&'a DistributionDensity>,
    Option<&'a DistributionPattern>,
    Option<&'a InstanceRotationYaw>,
    Option<&'a InstanceScale>,
    Option<&'a InstanceJitter>,
    &'a GlobalTransform,
);

pub fn handle_scatter_requests<TIn, TOut>(
    mut cmd: Commands,
    q_requests: Query<(Entity, &ScatterRequest<TIn, TOut>), With<ScatterRequest<TIn, TOut>>>,
    q_scatter_root: Query<
        (Entity, Option<&MapHeight>, &Aabb),
        (Without<ChunkRoot>, With<ScatterRoot>),
    >,
    q_chunk_root: Query<(Entity, &BaseChunkSize, Option<&MapHeight>, &Aabb), With<ChunkRoot>>,
    q_layer: Query<ScatterLayerQueryData, With<ScatterLayer>>,
    q_chunk: Query<
        (&ChunkSize, &GlobalTransform, &ChunkLevel, &ChunkCoord),
        (With<Chunk>, Without<Merging>),
    >,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    world_seed: Res<WorldSeed>,
    images: Res<Assets<Image>>,
) where
    TIn: Material + Send + 'static,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Send + 'static,
{
    let height_map_image = height_map.as_ref().and_then(|h| images.get(&h.0));
    let height_map_config = height_map_cfg.map(|cfg| cfg.into_inner());

    // NOTE: handle 2 per frame. TODO optimize / compute shaders for instanced procedural material
    for (entity, request) in q_requests.iter().take(2) {
        let Ok((
            scatter_root_ref,
            density_dist,
            pattern_dist,
            instance_rotation,
            instance_scale,
            instance_jitter,
            layer_gtf,
        )) = q_layer.get(request.layer_entity)
        else {
            warn!("ScatterLayer not found!");
            continue;
        };

        let density = density_dist.map_or(1.0, |d| **d);

        debug!(
            "Scattering {} instances in ScatterLayer {}",
            density, request.layer_entity,
        );

        let density_map_image = pattern_dist
            .and_then(|x| images.get(&x.density_map))
            .cloned();

        let task_data = if let Some(chunk_entity) = request.chunk_entity {
            let Ok((root_entity, base_chunk_size, map_height, aabb)) =
                q_chunk_root.get(**scatter_root_ref)
            else {
                warn!("ScatterRoot not found!");
                continue;
            };

            let Ok((chunk_size, chunk_gtf, chunk_level, chunk_coord)) = q_chunk.get(chunk_entity)
            else {
                continue;
            };

            let size = **base_chunk_size * Vec3::splat(**chunk_size as f32);

            let seed = generate_seed(&world_seed, chunk_coord);

            Some(ScatterTaskData {
                container: Container {
                    layer_entity: request.layer_entity,
                    chunk_entity: Some(chunk_entity),
                    root_entity,
                    instances_dim: density * (**chunk_level as f32 / 2. + 1.0),
                    corner: -size / 2.0,
                    height: 0.0,
                    size,
                    root_size: Vec3::from(aabb.half_extents * 2.),
                    transform: chunk_gtf.compute_transform(),
                    seed,
                },
                map_height: map_height.cloned(),
                scale: instance_scale.cloned(),
                rotation: instance_rotation.cloned(),
                jitter: instance_jitter.cloned(),
                height_map_image: height_map_image.cloned(),
                height_map_config: height_map_config.cloned(),
                density_map_image,
            })
        } else {
            let Ok((root_entity, map_height, aabb)) = q_scatter_root.get(**scatter_root_ref) else {
                warn!("ScatterRoot not found!");
                continue;
            };

            let size = Vec3::from(aabb.half_extents * 2.0);

            Some(ScatterTaskData {
                container: Container {
                    layer_entity: request.layer_entity,
                    chunk_entity: None,
                    root_entity,
                    instances_dim: density,
                    corner: -Vec3::from(aabb.half_extents),
                    height: 0.0,
                    size,
                    root_size: size,
                    transform: layer_gtf.compute_transform(),
                    seed: **world_seed,
                },
                map_height: map_height.cloned(),
                scale: instance_scale.cloned(),
                rotation: instance_rotation.cloned(),
                jitter: instance_jitter.cloned(),
                height_map_image: height_map_image.cloned(),
                height_map_config: height_map_config.cloned(),
                density_map_image,
            })
        };

        let Some(data) = task_data else {
            continue;
        };

        cmd.entity(entity).remove::<ScatterRequest<TIn, TOut>>();

        let task = AsyncComputeTaskPool::get()
            .spawn(async move { create_scatter_results_from_task_data::<TIn, TOut>(data) });

        cmd.entity(request.target_entity)
            .insert(CpuScatterTask(task));
    }
}

fn create_scatter_results_from_task_data<TIn, TOut>(
    task_data: ScatterTaskData,
) -> ScatterResults<TIn, TOut>
where
    TIn: Material + Send + 'static,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Send + 'static,
{
    let density_sampler = task_data
        .density_map_image
        .as_ref()
        .map(|x| DensityMapSampler::new(x, task_data.container.root_size));

    let height_sampler = task_data
        .height_map_config
        .as_ref()
        .and_then(|cfg| {
            task_data
                .height_map_image
                .as_ref()
                .map(|img| HeightMapSampler::Cpu(HeightMapCpuSampler::new(img, cfg)))
        })
        .unwrap_or(HeightMapSampler::Default(DefaultSampler));

    create_scatter_results(
        task_data.container,
        InstanceModifiers {
            jitter: task_data.jitter.as_ref(),
            map_height: task_data.map_height.as_ref(),
            height_sampler: &height_sampler,
            density_sampler: &density_sampler,
            scale: task_data.scale.as_ref(),
            rotation: task_data.rotation.as_ref(),
        },
    )
}

pub fn handle_finished_scatter_tasks<TIn, TOut>(
    mut cmd: Commands,
    mut tasks: Query<(Entity, &mut CpuScatterTask<ScatterResults<TIn, TOut>>)>,
    mut ew_results: EventWriter<ScatterResults<TIn, TOut>>,
    q_target: Query<Entity, Without<Merging>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    for (entity, mut task) in &mut tasks {
        let Some(results) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        if q_target.get(entity).is_err() {
            continue;
        }

        cmd.entity(entity)
            .remove::<CpuScatterTask<ScatterResults<TIn, TOut>>>();

        let mut targets = vec![results.root, results.layer];

        if let Some(chunk_entity) = results.chunk {
            targets.push(chunk_entity);
        }

        debug!("Scattered {} instances", results.data.len());

        cmd.trigger_targets(results.clone(), targets);
        ew_results.write(results);
    }
}
