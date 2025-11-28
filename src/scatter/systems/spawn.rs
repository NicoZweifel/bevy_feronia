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
) where
    T: ScatterMaterial,
{
    for event in mr_spawn.read() {
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

        let is_chunked =
            event.trigger.chunk.is_some() && q_scatter_chunked.get(event.trigger.layer).is_ok();

        let (chunk_gtf_translation, chunk_level) = event
            .trigger
            .chunk
            .and_then(|e| q_chunks.get(e).ok())
            .map(|(gtf, level)| (gtf.translation(), level.clone()))
            .unwrap_or_default();

        if is_chunked && q_chunks.get(event.trigger.chunk.unwrap()).is_err() {
            #[cfg(feature = "trace")]
            debug!(
                "Couldn't get chunk {:?}, it might've been despawned already or is in the process of merging!",
                event.trigger.chunk
            );
            continue;
        };

        let mut names: Vec<Name> = name_map.keys().cloned().collect();
        names.sort();

        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

        T::spawn(
            &mut cmd,
            SpawnRequest {
                event,
                chunk_level,
                chunk_gtf_translation,
                parent,
                lod_config,
                names,
                name_map,
                is_chunked,
            },
        );
    }
}
