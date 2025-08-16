use crate::prelude::*;
use bevy::camera::visibility::VisibilityRange;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rand::prelude::IteratorRandom;
use rand::rng;

pub fn spawn_extended_wind_affected(
    mut cmd: Commands,
    mut er_spawn: EventReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
    prototype_assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
    q_root: Query<(&LodConfig, Option<&ChunkLodConfig>), With<ScatterRoot>>,
    q_chunks: Query<&ChunkLevel, (With<Chunk>, Without<Merging>)>,
) {
    for e in er_spawn.read() {
        debug!("Spawning extended wind affected!");

        let mut chunk_level = ChunkLevel::default();

        if let Some(chunk) = e.trigger.chunk {
            let Ok(level) = q_chunks.get(chunk) else {
                continue;
            };

            chunk_level = level.clone();
        }

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

        let Ok((lod_config, chunk_lod_config)) = q_root.get(e.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        debug!(
            "Spawning {} extended prototypes in level {chunk_level:?}.",
            prototypes.len()
        );

        cmd.spawn_batch(
            e.trigger
                .data
                .clone()
                .iter_mut()
                .map(|scatter_result| {
                    let prototypes = name_map.values().choose(&mut rng());

                    let Some(prototypes) = prototypes else {
                        return vec![];
                    };

                    prototypes
                        .iter()
                        .map(|prototype| {
                            let parent = e.trigger.chunk.unwrap_or(e.trigger.layer);

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
                .flatten()
                .collect::<Vec<_>>(),
        );
    }
}
