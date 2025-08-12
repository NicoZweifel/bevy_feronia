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
) {
    for e in er_spawn.read() {
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
                        warn!("Couldn't choose Prototypes!");
                        return vec![];
                    };

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
                                VisibilityRange {
                                    start_margin: start_margin.clone(),
                                    end_margin: end_margin.clone(),
                                    use_aabb: false,
                                },
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect::<Vec<_>>(),
        );
    }
}
