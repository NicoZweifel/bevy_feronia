use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub fn merge(
    q_chunk: Query<(Entity, &ChildOf), (With<CanMerge>, With<Chunk>)>,
    mut ew_check: EventWriter<MergeCheck>,
) {
    let mut parents: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity, parent) in &q_chunk {
        parents.entry(parent.get()).or_default().push(entity);
    }

    for (parent, children) in parents {
        ew_check.write(MergeCheck { parent, children });
    }
}

pub fn handle_merge_check(
    q_center: Query<&GlobalTransform, With<ChunkCenter>>,
    q_chunk: Query<&MergeDistance, (With<CanMerge>, With<Chunk>)>,
    q_parent: Query<&GlobalTransform>,
    mut er_check: EventReader<MergeCheck>,
    mut ew_merge: EventWriter<MergeChunks>,
) {
    let Ok(center) = q_center.single() else {
        warn!(
            "Couldn't get ChunkCenter for merge! Did you forgot to add it to your Camera or Player entity?"
        );
        return;
    };

    let center = center.translation();

    for e in er_check.read() {
        if e.children.len() < 4 {
            continue;
        };

        let children = e.children.clone();
        let parent = e.parent;

        let first_child = children[0];

        let Ok(merge_distance) = q_chunk.get(first_child) else {
            warn!("Couldn't get MergeDistance for merge!");
            continue;
        };

        let parent_translation = q_parent.get(parent).unwrap().translation();
        let distance = center.distance(parent_translation);
        if distance < **merge_distance {
            continue;
        }

        ew_merge.write(MergeChunks { children, parent });
    }
}

pub fn handle_merge(mut cmd: Commands, mut er_merge: EventReader<MergeChunks>) {
    for e in er_merge.read() {
        let children = e.children.clone();
        let parent = e.parent;

        info!("Merging Chunks: {children:?} into {parent}");

        for child in &e.children {
            cmd.entity(*child).despawn();
        }

        cmd.entity(e.parent).insert(CanSplit);
    }
}
