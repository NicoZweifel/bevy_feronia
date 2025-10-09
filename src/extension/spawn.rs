use crate::prelude::*;
use crate::scatter::utils::select_consistent_prototype;
use bevy::prelude::*;

pub fn spawn_extended_wind_affected(
    mut cmd: Commands,
    mut mr_spawn: MessageReader<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
    prototype_assets: Res<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>,
    q_root: Query<&LodConfig, With<ScatterRoot>>,
    q_chunks: Query<&ChunkLevel, (With<Chunk>, Without<Merging>)>,
) {
    for event in mr_spawn.read() {
        debug!("Spawning extended wind affected!");

        let mut chunk_level = ChunkLevel::default();

        if let Some(chunk) = event.trigger.chunk {
            let Ok(level) = q_chunks.get(chunk) else {
                continue;
            };

            chunk_level = level.clone();
        }

        let Some((chosen_name, prototype)) = select_consistent_prototype(
            &event.items,
            event.trigger.seed,
            &prototype_assets,
            &chunk_level,
            event.trigger.chunk.is_some(),
        ) else {
            debug!(
                "No suitable extended prototype found for LOD {:?}. Skipping.",
                chunk_level
            );
            continue;
        };

        let Ok(lod_config) = q_root.get(event.trigger.root) else {
            warn!("Couldn't get ScatterRoot!");
            continue;
        };

        debug!("Spawning extended prototype '{chosen_name}' in level {chunk_level:?}");

        let parent = event.trigger.chunk.unwrap_or(event.trigger.layer);
        let visibility_range = lod_config.get_visibility_range(prototype.lod_level);

        let mesh_handle = prototype.mesh().clone();
        let material_handle = prototype.material().clone();

        cmd.spawn_batch(
            event
                .trigger
                .data
                .iter()
                .map(move |scatter_result| {
                    (
                        **scatter_result,
                        Mesh3d(mesh_handle.clone()),
                        MeshMaterial3d(material_handle.clone()),
                        WindAffected,
                        WindAffectedReady,
                        ChildOf(parent),
                        visibility_range.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
}
