use crate::core::*;
use crate::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::view::VisibilityRange;
use rand::prelude::IteratorRandom;
use rand::rng;

pub fn spawn_extended_wind_affected(
    mut cmd: Commands,
    mut er_spawn: EventReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
    prototype_assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
    q_root: Query<(&LodConfig, Option<&BaseChunkSize>), With<ScatterRoot>>,
    q_chunks: Query<(&GlobalTransform, &ChunkLevel), With<Chunk>>,
) {
    for e in er_spawn.read() {
        debug!("Spawning extended wind affected!");

        let (chunk_gtf, chunk_level) = e
            .trigger
            .chunk
            .map(|x| {
                q_chunks.get(x).ok().map(|(chunk_gtf, chunk_level)| {
                    (chunk_gtf.compute_transform(), chunk_level.clone())
                })
            })
            .flatten()
            .unwrap_or_else(|| (Transform::default(), ChunkLevel::default()));

        let mut prototypes: Vec<ScatterAsset<ExtendedWindAffectedMaterial>> = vec![];
        for item in e.items.iter() {
            let prototype = prototype_assets.get(&item.0);

            let Some(prototype) = prototype else {
                warn!("Couldn't get ScatterRoot!");
                return;
            };

            prototypes.push(prototype.clone());
        }

        let mut name_map = HashMap::<Name, Vec<&ScatterAsset<ExtendedWindAffectedMaterial>>>::new();

        prototypes
            .iter()
            .filter(|x| e.trigger.chunk.is_none() || *x.lod_level == *chunk_level)
            .map(|x| (x.name.clone().unwrap_or(Name::new("")), x))
            .for_each(|(name, x)| {
                name_map
                    .get_mut(&name)
                    .map(|y| y.push(x))
                    .map(|_| x)
                    .or_else(|| name_map.insert(name, vec![x]).map(|_| x));
            });

        let Ok((lod_config, base_chunk_size)) = q_root.get(e.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        cmd.spawn_batch(
            e.trigger
                .data
                .clone()
                .iter_mut()
                .map(|result| {
                    let prototypes = name_map.values().choose(&mut rng());

                    let Some(prototypes) = prototypes else {
                        return vec![];
                    };

                    debug!("Spawning {} prototypes.", prototypes.len());

                    prototypes
                        .iter()
                        .map(|prototype| {
                            let lod_level = prototype.lod_level;

                            const FADE_BAND: f32 = 2.0;

                            let current_lod_dist = lod_config
                                .get(*lod_level as usize)
                                .map_or(*LodLevelDistance::default(), |x| **x);

                            let start_margin = if *lod_level == 0 {
                                0.0..0.0
                            } else {
                                let prev_lod_dist = *(**lod_config)[*lod_level as usize - 1];
                                prev_lod_dist - FADE_BAND..prev_lod_dist
                            };

                            let end_margin = if *lod_level == lod_config.get_max_lod_level() {
                                f32::MAX..f32::MAX
                            } else {
                                current_lod_dist - FADE_BAND..current_lod_dist
                            };

                            (
                                Mesh3d(prototype.mesh().clone()),
                                MeshMaterial3d(prototype.material().clone()),
                                **result,
                                WindAffected,
                                WindAffectedReady,
                                ChildOf(e.trigger.chunk.unwrap_or(e.trigger.layer)),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>(),
        );
    }
}
