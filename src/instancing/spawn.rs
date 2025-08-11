use crate::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use rand::prelude::IteratorRandom;
use rand::rng;

pub fn spawn_instanced_wind_affected(
    mut er_spawn: EventReader<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
    mut cmd: Commands,
    prototype_assets: Res<Assets<ScatterAsset<InstancedWindAffectedMaterial>>>,
    q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
) {
    for e in er_spawn.read() {
        let instances = e
            .trigger
            .data
            .iter()
            .enumerate()
            .map(|(i, res)| InstanceData {
                position: res.translation,
                scale: res.scale.element_sum() / 3.0,
                // TODO expose
                color: LinearRgba::from(Color::hsla(78., 0.98, 0.5, 1.0)).to_f32_array(),
                index: i as u32,
            })
            .collect::<Vec<_>>();

        // TODO
        let (chunk_gtf, chunk_root, chunk_level) =
            e.trigger
                .chunk
                .map_or((Transform::default(), None, ChunkLevel::default()), |x| {
                    q_chunks.get(x).ok().map_or(
                        (Transform::default(), None, ChunkLevel::default()),
                        |(chunk_gtf, chunk_root, chunk_level)| {
                            (
                                chunk_gtf.compute_transform(),
                                Some(chunk_root),
                                chunk_level.clone(),
                            )
                        },
                    )
                });

        let mut prototypes: Vec<ScatterAsset<InstancedWindAffectedMaterial>> = vec![];
        for item in e.items.iter() {
            let prototype = prototype_assets.get(&item.0);

            let Some(prototype) = prototype else {
                warn!("Couldn't get ScatterRoot!");
                return;
            };

            prototypes.push(prototype.clone());
        }

        let mut name_map =
            HashMap::<Name, Vec<&ScatterAsset<InstancedWindAffectedMaterial>>>::new();

        prototypes
            .iter()
            .filter(|x| *x.lod_level >= *chunk_level)
            .map(|x| (x.name.clone().unwrap_or(Name::new("")), x))
            .for_each(|(name, x)| {
                name_map
                    .get_mut(&name)
                    .map(|y| y.push(x))
                    .map(|_| x)
                    .or_else(|| name_map.insert(name, vec![x]).map(|_| x));
            });

        let prototypes = name_map.values().choose(&mut rng());

        let Some(prototypes) = prototypes else {
            warn!("Couldn't choose Prototypes!");
            return;
        };

        info!("Spawning {} prototypes.", prototypes.len());

        let Ok(lod_config) = q_root.get(e.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        for prototype in prototypes {
            let lod_level = prototype.lod_level;

            let mesh_handle = prototype.mesh().clone();
            let (mut min_point, mut max_point) = (Vec3::MAX, Vec3::MIN);

            let instances = instances
                .iter()
                .map(|instance| {
                    let mut instance = instance.clone();

                    instance.position = instance.position + chunk_gtf.translation.with_y(0.);

                    let instance_min =
                        instance.position + Vec3::from(prototype.aabb().min() * instance.scale);
                    let instance_max =
                        instance.position + Vec3::from(prototype.aabb().max() * instance.scale);
                    min_point = min_point.min(instance_min);
                    max_point = max_point.max(instance_max);

                    instance
                })
                .collect::<Vec<_>>();

            let entity = cmd
                .spawn((
                    InstancedWindAffectedMeshMaterial(prototype.material().clone()),
                    Mesh3d(mesh_handle),
                    InstanceMaterialData(instances.clone()),
                    NoAutomaticBatching,
                    WindAffected,
                    WindAffectedReady,
                ))
                .id();

            // TODO
            // let lod_level = *chunk_level as usize;
            let lod_level = *lod_level as usize;

            let current_lod_dist = lod_config
                .get(lod_level)
                .map_or(*LodLevelDistance::default(), |x| **x);

            const FADE_BAND: f32 = 2.0;

            let start_margin = if lod_level == 0 {
                0.0..0.0
            } else {
                let prev_lod_dist = *(**lod_config)[lod_level - 1];
                prev_lod_dist - FADE_BAND..prev_lod_dist
            };

            let end_margin = if lod_level as u32 == lod_config.get_max_lod_level() {
                f32::MAX..f32::MAX
            } else {
                current_lod_dist - FADE_BAND..current_lod_dist
            };

            let chunk_center = chunk_gtf.translation;

            let local_min = min_point - chunk_center;
            let local_max = max_point - chunk_center;

            let local_aabb = Aabb::from_min_max(local_min, local_max);
            cmd.entity(entity).insert((
                Aabb::from(local_aabb),
                /* VisibilityRange {
                    start_margin,
                    end_margin,
                    use_aabb: false,
                },*/
                ChildOf(e.trigger.chunk.unwrap_or(e.trigger.target)),
            ));
        }
    }
}
