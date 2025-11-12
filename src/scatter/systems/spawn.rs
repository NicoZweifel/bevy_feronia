use crate::core::SpawnScatterAssets;
use crate::prelude::*;
use bevy::asset::Assets;
use bevy::pbr::Material;
use bevy::prelude::*;

pub fn spawn<TOut, TIn>(
    mut cmd: Commands,
    mut mr_spawn: MessageReader<SpawnScatterAssets<TOut>>,
    prototype_assets: Res<Assets<ScatterAsset<TOut>>>,
    q_chunks: Query<(&GlobalTransform, &ChunkLevel), (With<Chunk>, Without<Merging>)>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
    q_layers: Query<(), With<ScatterChunked>>,
) where
    TOut: ScatterMaterial<TIn>,
    TIn: Material,
{
    for event in mr_spawn.read() {
        let Ok(lod_config) = q_root.get(event.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            continue;
        };

        let name_map = &event.create_name_map(&prototype_assets);
        if name_map.is_empty() {
            warn!("No assets found for spawn event!");
            continue;
        }

        let (chunk_gtf_translation, chunk_level) = event
            .trigger
            .chunk
            .and_then(|e| q_chunks.get(e).ok())
            .map(|(gtf, level)| (gtf.translation(), level.clone()))
            .unwrap_or_default();

        let is_chunked = event.trigger.chunk.is_some() && q_layers.get(event.trigger.layer).is_ok();

        let mut names: Vec<Name> = name_map.keys().cloned().collect();
        names.sort();

        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);

        TOut::spawn(
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
