use std::hash::Hasher;
use std::hash::Hash;
use crate::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::prelude::IteratorRandom;
use rand::{rng, SeedableRng};
use rand_pcg::Pcg64;

pub fn spawn_extended_wind_affected(
    mut cmd: Commands,
    mut er_spawn: EventReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
    prototype_assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
    q_chunks: Query<&ChunkLevel, (With<Chunk>, Without<Merging>)>,
) {
    for event in er_spawn.read() {
        debug!("Spawning extended wind affected!");

        let mut chunk_level = ChunkLevel::default();

        if let Some(chunk) = event.trigger.chunk {
            let Ok(level) = q_chunks.get(chunk) else {
                continue;
            };

            chunk_level = level.clone();
        }

        let mut prototypes: Vec<ScatterAsset<ExtendedWindAffectedMaterial>> = vec![];
        for item in event.items.iter() {
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
            .filter(|x| event.trigger.chunk.is_none() || *x.lod_level == *chunk_level)
            .map(|x| (x.name.clone().unwrap_or(Name::new("")), x))
            .for_each(|(name, x)| {
                name_map
                    .get_mut(&name)
                    .map(|y| y.push(x))
                    .map(|_| x)
                    .or_else(|| name_map.insert(name, vec![x]).map(|_| x));
            });

        let Ok(lod_config) = q_root.get(event.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        debug!(
            "Spawning {} extended prototypes in level {chunk_level:?}.",
            prototypes.len()
        );

        cmd.spawn_batch(
            event.trigger
                .data
                .clone()
                .iter_mut()
                .flat_map(|scatter_result| {
                   let mut instance_hasher = std::collections::hash_map::DefaultHasher::new();

                    event.trigger.seed.hash(&mut instance_hasher);
                    scatter_result.hash(&mut instance_hasher);

                    let instance_seed = instance_hasher.finish();

                    let mut rng = Pcg64::seed_from_u64(instance_seed);

                    let prototypes = name_map.values().choose(&mut rng);

                    let Some(prototypes) = prototypes else {
                        return vec![];
                    };

                    prototypes
                        .iter()
                        .map(|prototype| {
                            let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

                            let visibility_range =
                                lod_config.get_visibility_range(prototype.lod_level);

                            (
                                **scatter_result,
                                Mesh3d(prototype.mesh().clone()),
                                MeshMaterial3d(prototype.material().clone()),
                                WindAffected,
                                WindAffectedReady,
                                ChildOf(parent),
                                visibility_range,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        );
    }
}
