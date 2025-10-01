use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub fn merge_check(
    q_chunk: Query<
        (Entity, &ChildOf),
        (
            With<CanMerge>,
            Without<Merging>,
            With<Chunk>,
            Without<ChunkInitialize>,
        ),
    >,
    mut mw_check: MessageWriter<MergeCheck>,
) {
    let mut parents: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity, parent) in &q_chunk {
        parents.entry(parent.get()).or_default().push(entity);
    }

    for (parent, children) in parents {
        mw_check.write(MergeCheck { parent, children });
    }
}

pub fn handle_merge_check(
    mut cmd: Commands,
    q_center: Query<&GlobalTransform, With<ChunkCenter>>,
    q_chunk: Query<&MergeDistance, (With<CanMerge>, Without<Merging>, With<Chunk>)>,
    q_parent: Query<&GlobalTransform, With<Chunk>>,
    mut mr_check: MessageReader<MergeCheck>,
) {
    let Ok(center) = q_center.single() else {
        warn!(
            "Couldn't get ChunkCenter for merge! Did you forgot to add it to your Camera or Player entity?"
        );
        return;
    };

    let center = center.translation();

    for e in mr_check.read() {
        let parent = e.parent;
        let Ok(parent_tf) = q_parent.get(parent) else {
            warn!("Couldn't get parent Chunk for merge!");
            continue;
        };

        let children = e.children.clone();

        let first_child = children[0];

        let Ok(merge_distance) = q_chunk.get(first_child) else {
            warn!("Couldn't get Chunk {first_child} in level for merge!");
            continue;
        };

        // TODO don't hardcode but use ChunkSizeScalarConfig / ChunkRootSizeDim
        if e.children.len() < 4 {
            continue;
        };

        let distance = center.distance(parent_tf.translation());
        if distance < **merge_distance {
            continue;
        }

        for child in &e.children {
            cmd.entity(*child).insert(Merging);
        }
    }
}

pub fn merge(
    q_chunk: Query<(Entity, &ChildOf), (With<Merging>, With<Chunk>)>,
    mut mw_merge: MessageWriter<MergeChunks>,
) {
    let mut merge_chunks_map = HashMap::<Entity, Vec<Entity>>::new();

    for (child, parent) in q_chunk {
        if let Some(children) = merge_chunks_map.get_mut(&parent.0) {
            children.push(child);
            continue;
        }

        merge_chunks_map.insert(parent.0, vec![child]);
    }

    for (parent, children) in &mut merge_chunks_map {
        mw_merge.write(MergeChunks {
            children: children.clone(),
            parent: *parent,
        });
    }
}

pub fn handle_merge(mut cmd: Commands, mut mr_merge: MessageReader<MergeChunks>) {
    for e in mr_merge.read() {
        let children = e.children.clone();
        let parent = e.parent;

        debug!("Merging Chunks: {children:?} into {parent}");

        for child in &e.children {
            cmd.entity(*child).despawn();
        }

        cmd.entity(e.parent).insert(CanSplit);
    }
}
