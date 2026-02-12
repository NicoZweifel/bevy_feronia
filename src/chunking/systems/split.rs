use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::{IVec2, Vec3};
use bevy_transform::prelude::{GlobalTransform, Transform};

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub fn split(
    q_center: Query<&GlobalTransform, With<Center>>,
    q_chunk: Query<
        (Entity, &GlobalTransform, &SplitDistance),
        (With<CanSplit>, With<Chunk>, Without<Merging>),
    >,
    mut mw_split: MessageWriter<SplitChunk>,
) {
    let Ok(center) = q_center.single() else {
        #[cfg(feature = "trace")]
        debug!(
            "Couldn't get ChunkCenter for split! Did you forgot to add it to your Camera or Player entity?"
        );
        return;
    };

    let center = center.translation();

    for entity in q_chunk
        .iter()
        .filter_map(|(entity, chunk_transform, split_distance)| {
            let distance = center.distance(chunk_transform.translation());
            let check = distance < **split_distance;

            check.then_some(entity)
        })
    {
        mw_split.write(SplitChunk(entity));
    }
}

pub fn handle_split(
    mut cmd: Commands,
    mut mr_split: MessageReader<SplitChunk>,
    q_chunk: Query<(&ChunkLevel, &ChunkSize, &ChunkOf, &ChunkCoord)>,
    q_chunk_config: Query<(&BaseChunkSize, &ChunkLodConfig, &ChunkSizeScalarConfig)>,
) {
    for e in mr_split.read() {
        let parent_entity = e.get();
        #[cfg(feature = "trace")]
        debug!("Splitting Chunk: {parent_entity}");

        let Ok((parent_chunk_level, parent_chunk_size, root_chunk, parent_coord)) =
            q_chunk.get(parent_entity)
        else {
            #[cfg(feature = "trace")]
            debug!("Couldn't get Chunk for split: {parent_entity}");
            continue;
        };

        let (base_chunk_size, lod_cfg, scalar_cfg) = q_chunk_config.get(**root_chunk).unwrap();

        let parent_level = **parent_chunk_level;
        if parent_level == 0 {
            #[cfg(feature = "trace")]
            warn!("Can't split root chunk!");
            continue;
        }

        let child_level = parent_level - 1;

        let child_chunk_size = **scalar_cfg.get_scalar_config(child_level);
        let subdivision_scalar = **parent_chunk_size / child_chunk_size;

        let parent_world_size = **parent_chunk_size as f32 * **base_chunk_size;
        let child_world_size = parent_world_size / subdivision_scalar as f32;

        let half_parent_size = parent_world_size / 2.0;
        let half_child_size = child_world_size / 2.0;

        let child_chunk_size = **scalar_cfg.get_scalar_config(child_level);

        for (x, z) in
            (0..subdivision_scalar).flat_map(|z| (0..subdivision_scalar).map(move |x| (x, z)))
        {
            let offset = Vec3::new(
                (x as f32 * child_world_size.x) - half_parent_size.x + half_child_size.x,
                0.0,
                (z as f32 * child_world_size.z) - half_parent_size.z + half_child_size.z,
            );

            let child_coord = IVec2::new(
                parent_coord.0.x * subdivision_scalar as i32 + x as i32,
                parent_coord.0.y * subdivision_scalar as i32 + z as i32,
            );

            let child_entity = cmd
                .spawn((
                    Chunk,
                    ChunkSize(child_chunk_size),
                    ChunkLevel(child_level),
                    Transform::from_translation(offset),
                    ChunkOf(**root_chunk),
                    ChildOf(parent_entity),
                    ChunkCoord(child_coord),
                ))
                .id();

            if child_level > 0 {
                let child_lod_config = lod_cfg.get_lod_config(child_level - 1);
                cmd.entity(child_entity)
                    .insert(SplitDistance(*child_lod_config));
            }

            if child_level < lod_cfg.get_max_lod() {
                cmd.entity(child_entity)
                    .insert(MergeDistance(*lod_cfg.get_lod_config(child_level)));
            }
        }

        cmd.entity(parent_entity).remove::<CanSplit>();
    }
}
