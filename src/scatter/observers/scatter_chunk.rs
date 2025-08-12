use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

type LayerQueryItem<'a> = (
    Entity,
    &'a ScatterLayerOf,
    Option<&'a DistributionDensity>,
    Option<&'a DistributionPattern>,
    Option<&'a InstanceRotationYaw>,
    Option<&'a InstanceScale>,
    Option<&'a InstanceJitter>,
);

pub fn scatter_chunk<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut trigger: On<ScatterChunk<TIn, TOut>>,
    mut cmd: Commands,
    height_map_cfg: Option<Res<HeightMapConfig>>,
    height_map: Option<Res<HeightMap>>,
    images: Res<Assets<Image>>,
    q_root: Query<(Entity, &BaseChunkSize, Option<&MapHeight>, &Aabb), With<ChunkRoot>>,
    q_layer: Query<LayerQueryItem, (With<ScatterLayer>, With<ScatterLayerType<TIn, TOut>>)>,
    q_chunk: Query<(Entity, &ChunkSize, &GlobalTransform), With<Chunk>>,
    mut ew_results: EventWriter<ScatterResults<TIn, TOut>>,
) {
    trigger.propagate(false);

    let height_sampler = get_height_map_sampler(&images, height_map_cfg, height_map);

    let Ok((layer_entity, scatter_root, density_dist, pattern_dist, rotation, scale, jitter)) =
        q_layer.get(trigger.scatter_layer)
    else {
        warn!("ScatterLayer not found!");
        return;
    };

    let Ok((root, base_chunk_size, map_height, aabb)) = q_root.get(**scatter_root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    let density_sampler = get_density_sampler(pattern_dist, &images, *aabb);

    let instances_dim = density_dist.map_or(10., |d| **d);

    info!(
        "Scattering {} instances in Chunk {}",
        (instances_dim as u32).pow(2),
        trigger.target()
    );

    let Ok((chunk_entity, chunk_size, chunk_gtf)) = q_chunk.get(trigger.target()) else {
        warn!("Chunk not found!");
        return;
    };

    let size = **base_chunk_size * Vec3::splat(**chunk_size as f32);

    let corner = -size / 2.;

    let results = create_scatter_results(
        Container {
            layer_entity,
            chunk_entity: Some(chunk_entity),
            instances_dim,
            corner,
            height: 0.,
            size,
            transform: chunk_gtf.compute_transform(),
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

    info!(
        "Scattered {} instances in Chunk {}",
        results.data.len(),
        trigger.target()
    );

    cmd.trigger_targets(results.clone(), [root, layer_entity, trigger.target()]);
    ew_results.write(results);
}
