use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_asset::Assets;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_image::Image;
use bevy_math::Vec3;
use bevy_tasks::AsyncComputeTaskPool;
use bevy_tasks::futures_lite::future;
use bevy_transform::prelude::GlobalTransform;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

#[derive(QueryData)]
#[query_data()]
pub struct ScatterLayerQueryData {
    scatter_root: &'static ScatterLayerOf,
    density_dist: Option<&'static DistributionDensity>,
    pattern_dist: Option<&'static DistributionPattern>,
    instance_rotation: Option<&'static InstanceRotationYawRange>,
    instance_scale: Option<&'static InstanceScaleRange>,
    instance_jitter: Option<&'static InstanceJitterStrength>,
    avoidance: Option<&'static Avoidance>,
    scale_density: Option<&'static ScaleDensity>,
    lod_config: Option<&'static LodConfig>,
    layer_gtf: &'static GlobalTransform,
}

// TODO refactor/split this up
pub fn handle_scatter_requests<T>(
    mut cmd: Commands,
    q_requests: Query<(Entity, &ScatterRequest<T>)>,
    q_scatter_root: Query<(Entity, Option<&MapHeight>, &Aabb), With<ScatterRoot>>,
    q_chunk_root: Query<
        (
            Entity,
            &BaseChunkSize,
            Option<&MapHeight>,
            &Aabb,
            &LodConfig,
        ),
        With<ChunkRoot>,
    >,
    q_layer: Query<ScatterLayerQueryData, With<ScatterLayer>>,
    q_chunk: Query<
        (&ChunkSize, &GlobalTransform, &ChunkLevel, &ChunkCoord),
        (With<Chunk>, Without<Merging>),
    >,
    mut q_scatter_state: Query<&mut HierarchicalScatterState<T>, With<ScatterRoot>>,
    q_occupancy_map: Query<&ScatterOccupancyMap, With<ScatterRoot>>,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    world_seed: Res<WorldSeed>,
    images: Res<Assets<Image>>,
) where
    T: ScatterMaterial,
{
    let height_map_image = height_map.as_ref().and_then(|h| images.get(&h.0));
    let height_map_config = height_map_cfg.map(|cfg| cfg.into_inner());

    // NOTE: handle 1 per tick/frame. TODO optimize / create compute pipeline for this.
    for (entity, request) in q_requests.iter().take(1) {
        let layer = request.layer_entity;

        let Ok(ScatterLayerQueryDataItem {
            scatter_root,
            density_dist,
            pattern_dist,
            instance_rotation,
            instance_scale,
            instance_jitter,
            avoidance,
            scale_density,
            layer_gtf,
            lod_config,
        }) = q_layer.get(layer)
        else {
            #[cfg(feature = "trace")]
            debug!("ScatterLayer {layer} not found!");
            continue;
        };

        let scatter_root = **scatter_root;
        let Ok(mut scatter_state) = q_scatter_state.get_mut(scatter_root) else {
            #[cfg(feature = "trace")]
            debug!("ScatterRoot {scatter_root} state not found!");
            continue;
        };

        let Ok(occupancy_map) = q_occupancy_map.get(scatter_root) else {
            #[cfg(feature = "trace")]
            debug!("ScatterRoot {scatter_root} occupancy not found!");
            continue;
        };

        let density = density_dist.map_or(1.0, |d| **d);

        #[cfg(feature = "trace")]
        debug!(
            "Scattering {density} instances in ScatterLayer {}",
            request.layer_entity
        );

        let density_map_image = pattern_dist.and_then(|p| images.get(&**p)).cloned();

        let task_data = if let Some(chunk) = request.chunk_entity {
            let Ok((root_entity, base_chunk_size, map_height, aabb, root_lod_config)) =
                q_chunk_root.get(scatter_root)
            else {
                #[cfg(feature = "trace")]
                warn!("ChunkRoot {} not found!", scatter_root);
                continue;
            };

            let Ok((chunk_size, chunk_gtf, chunk_level, chunk_coord)) = q_chunk.get(chunk) else {
                #[cfg(feature = "trace")]
                warn!("Chunk {chunk} not found!");
                continue;
            };

            let size = **base_chunk_size * Vec3::splat(**chunk_size as f32);

            let seed = generate_seed(&world_seed, chunk_coord);

            let instances_dim = density * (**chunk_level as f32 * 2.).max(1.);

            Some(ScatterTaskData {
                container: Container {
                    entity: request.target_entity,
                    layer_entity: request.layer_entity,
                    chunk_entity: Some(chunk),
                    root_entity,
                    instances_dim,
                    corner: -size / 2.0,
                    height: 0.0,
                    size,
                    root_size: Vec3::from(aabb.half_extents * 2.),
                    global_transform: *chunk_gtf,
                    root_global_transform: *layer_gtf,
                    seed,
                },
                map_height: map_height.cloned(),
                scale: instance_scale.cloned(),
                rotation: instance_rotation.cloned(),
                jitter: instance_jitter.cloned(),
                avoidance: avoidance.cloned(),
                external_avoidance_data: occupancy_map.clone(),
                density: scale_density.map(|_| {
                    lod_config
                        .unwrap_or(root_lod_config)
                        .density
                        .get(**chunk_level as usize)
                        .cloned()
                        .unwrap_or_default()
                }),
                height_map_image: height_map_image.cloned(),
                height_map_config: height_map_config.cloned(),
                density_map_image,
            })
        } else {
            let Ok((root_entity, map_height, aabb)) = q_scatter_root.get(scatter_root) else {
                #[cfg(feature = "trace")]
                debug!("ScatterRoot {} not found!", scatter_root);
                continue;
            };

            let size = Vec3::from(aabb.half_extents * 2.0);

            Some(ScatterTaskData {
                container: Container {
                    entity: request.target_entity,
                    layer_entity: request.layer_entity,
                    chunk_entity: None,
                    root_entity,
                    instances_dim: density,
                    corner: Vec3::from(aabb.center) - Vec3::from(aabb.half_extents),
                    height: 0.0,
                    size,
                    root_size: size,
                    global_transform: *layer_gtf,
                    root_global_transform: *layer_gtf,
                    seed: **world_seed,
                },
                map_height: map_height.cloned(),
                scale: instance_scale.cloned(),
                rotation: instance_rotation.cloned(),
                jitter: instance_jitter.cloned(),
                avoidance: avoidance.cloned(),
                external_avoidance_data: occupancy_map.clone(),
                height_map_image: height_map_image.cloned(),
                height_map_config: height_map_config.cloned(),
                density: None,
                density_map_image,
            })
        };

        let Some(data) = task_data else {
            continue;
        };

        cmd.entity(entity).remove::<ScatterRequest<T>>();

        let task =
            AsyncComputeTaskPool::get().spawn(async move { ScatterResults::<T>::from(data) });

        cmd.entity(request.target_entity)
            .insert(CpuScatterTask(task));

        scatter_state.pending_tasks += 1;
    }
}

pub fn handle_finished_scatter_tasks<T>(
    mut cmd: Commands,
    mut tasks: Query<(Entity, &mut CpuScatterTask<ScatterResults<T>>)>,
    mut mw_results: MessageWriter<ScatterResults<T>>,
    q_target: Query<Entity, Without<Merging>>,
) where
    T: ScatterMaterial,
{
    for (entity, mut task) in &mut tasks {
        let Some(results) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        if q_target.get(entity).is_err() {
            continue;
        }

        cmd.entity(entity)
            .remove::<CpuScatterTask<ScatterResults<T>>>();

        let mut targets = vec![results.root, results.layer];

        if let Some(chunk_entity) = results.chunk {
            targets.push(chunk_entity);
        }

        #[cfg(feature = "trace")]
        debug!("Scattered {} instances", results.data.len());

        targets
            .into_iter()
            .map(|entity| {
                let mut results = results.clone();
                results.entity = entity;
                results
            })
            .for_each(|results| cmd.trigger(results));

        mw_results.write(results);
    }
}
