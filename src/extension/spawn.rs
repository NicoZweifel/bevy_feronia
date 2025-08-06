use crate::core::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::view::VisibilityRange;

pub fn spawn_extended_wind_affected<T, P>(
    mut cmd: Commands,
    mut er_spawn: EventReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
    prototypes: Res<T>,
    prototype_assets: Res<Assets<P>>,
    // TODO use chunks if spawned for chunk
    _q_chunks: Query<(&GlobalTransform, &ChunkOf, &ChunkLevel), With<Chunk>>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
) where
    T: Resource + ProtoTypes<ExtendedWindAffectedMaterial, P>,
    P: ProtoType<ExtendedWindAffectedMaterial> + Asset + Clone,
{
    for e in er_spawn.read() {
        let Some(prototypes) = prototypes.choose(&e.items) else {
            warn!("Couldn't choose prototype!");
            return;
        };

        let Ok(lod_config) = q_root.get(e.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            return;
        };

        info!("Spawning {} prototypes.", prototypes.len());

        for (lod_level, prototype) in prototypes {
            let Some(prototype) = prototype_assets.get(&prototype) else {
                warn!("Couldn't get prototype!");
                return;
            };

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

            cmd.spawn_batch(
                e.trigger
                    .data
                    .iter()
                    .map(|result| {
                        (
                            Mesh3d(prototype.mesh().clone()),
                            MeshMaterial3d(prototype.material().clone()),
                            **result,
                            WindAffected,
                            WindAffectedReady,
                            ChildOf(e.trigger.layer),
                            VisibilityRange {
                                start_margin: start_margin.clone(),
                                end_margin: end_margin.clone(),
                                use_aabb: false,
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
}
