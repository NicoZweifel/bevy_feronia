use bevy::ecs::relationship::Relationship;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use crate::prelude::*;

pub fn merge(
    cfg: Res<ChunkConfig>,
    q_center: Query<&GlobalTransform, With<ChunkCenter>>,
    q_chunk: Query<(Entity, &Chunk, &ChildOf), With<CanMerge>>,
    q_parent: Query<&GlobalTransform>,
    mut ew_merge: EventWriter<MergeChunks>,
) {
    let Ok(center) = q_center.single() else {
        return;
    };

    let center_translation = center.translation();
    let max_lod_level = cfg.get_max_lod_level();
    let mut potential_parents: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity, chunk, parent) in &q_chunk {
        if chunk.level >= max_lod_level {
            continue;
        };

        potential_parents
            .entry(parent.get())
            .or_default()
            .push(entity);
    }

    for (parent, siblings) in potential_parents {
        if siblings.len() < 4 {
            continue;
        };

        let chunk_level = q_chunk.get(siblings[0]).unwrap().1.level;
        let merge_dist = cfg.lods[chunk_level as usize].distance;

        let parent_translation = q_parent.get(parent).unwrap().translation();

        if center_translation.distance(parent_translation) <= merge_dist {
            continue;
        }

        ew_merge.write(MergeChunks { siblings });
    }
}


pub fn handle_merge(mut cmd: Commands, mut er_merge: EventReader<MergeChunks>) {
    for e in er_merge.read() {
        for sibling_entity in &e.siblings {
            cmd.entity(*sibling_entity).despawn();
        }
    }
}
