use crate::prelude::events::SpawnScatterAssets;
use crate::prelude::*;

use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_transform::prelude::GlobalTransform;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub fn spawn<T>(
    mut cmd: Commands,
    mut mr_spawn: MessageReader<SpawnScatterAssets<T>>,
    prototype_assets: Res<Assets<ScatterAsset<T>>>,
    q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
    q_scatter_chunked: Query<(), With<ScatterChunked>>,
    q_transforms: Query<&GlobalTransform>,
    q_target: Query<Entity, Without<Merging>>,
) where
    T: ScatterMaterial,
{
    for event in mr_spawn.read() {
        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);
        if q_target.get(parent).is_err() {
            return;
        }

        let Ok(lod_config) = q_root.get(event.trigger.root) else {
            #[cfg(feature = "trace")]
            warn!("Couldn't get ScatterRoot!");
            continue;
        };

        let name_map = &event.create_name_map(&prototype_assets);
        if name_map.is_empty() {
            #[cfg(feature = "trace")]
            warn!("No assets found for spawn event!");
            continue;
        }

        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

        let is_chunked =
            event.trigger.chunk.is_some() && q_scatter_chunked.get(event.trigger.layer).is_ok();

        let (container_gtf, chunk_level) = event
            .trigger
            .chunk
            .and_then(|c| {
                q_chunks
                    .get(c)
                    .inspect_err(|_| {
                        #[cfg(feature = "trace")]
                        debug!(
                            "Couldn't get chunk {:?}, \
                            it might've been despawned already or is in the process of merging!",
                            event.trigger.chunk
                        );
                    })
                    .ok()
            })
            .map(|(x, y)| (*x, *y))
            .unwrap_or_else(|| {
                (
                    q_transforms
                        .get(parent)
                        .cloned()
                        .unwrap_or(GlobalTransform::IDENTITY),
                    ChunkLevel::default(),
                )
            });

        let mut names: Vec<Name> = name_map.keys().cloned().collect();
        names.sort();

        T::spawn(
            &mut cmd,
            SpawnRequest {
                event,
                chunk_level,
                container_gtf,
                parent,
                lod_config,
                names,
                name_map,
                is_chunked,
            },
        );
    }
}
