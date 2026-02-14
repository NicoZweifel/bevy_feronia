use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;
use bevy_ecs::relationship::Relationship;
use bevy_platform::collections::HashMap;
use bevy_transform::prelude::GlobalTransform;

use crate::utils::despawn;
#[cfg(feature = "trace")]
use tracing::{debug, error, warn};

pub fn merge_check(
    q_chunk: Query<
        (Entity, &ChildOf),
        (
            With<Chunk>,
            With<CanMerge>,
            With<MergeDistance>,
            Without<Merging>,
            Without<ChunkInitialize>,
        ),
    >,
    q_root: Query<&ChunkRootSizeDim, With<ChunkRoot>>,
    q_parent: Query<&ChunkOf, With<Chunk>>,
    mut mw_merge_check: MessageWriter<MergeCheck>,
) {
    let batch_iter = get_parent_child_map(q_chunk)
        .into_iter()
        .filter_map(|(parent, children)| {
            let root = q_parent
                .get(parent)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    error!("Couldn't get parent Chunk {parent} for merge check!");
                })
                .ok()?;

            let root_size_dim = q_root
                .get(**root)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    error!("Couldn't get ChunkRoot {} for merge check!", **root);
                })
                .ok()?;

            let check = children.len() >= root_size_dim.pow(2) as usize;
            check.then(|| MergeCheck::new(parent, children))
        });

    mw_merge_check.write_batch(batch_iter);
}

pub fn handle_merge_check(
    mut cmd: Commands,
    q_center: Query<&GlobalTransform, With<Center>>,
    q_merge_distance: Query<&MergeDistance>,
    q_parent: Query<&GlobalTransform>,
    mut mr_merge_check: MessageReader<MergeCheck>,
) {
    let Ok(center) = q_center.single() else {
        #[cfg(feature = "trace")]
        trace!(
            "Couldn't get Center for merge! Did you forgot to add it to your Camera or Player entity?"
        );
        return;
    };

    let center = center.translation();

    for child in mr_merge_check
        .read()
        .filter_map(|merge_check| {
            let MergeCheck {
                parent, children, ..
            } = merge_check;
            let first_child = children[0];

            let parent_tf = q_parent
                .get(*parent)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!("Couldn't get parent transform for merge check!")
                })
                .ok()?;

            let merge_distance = q_merge_distance
                .get(first_child)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!("Couldn't get merge distance for merge check! Was this Chunk already removed?")
                })
                .ok()?;

            merge_check
                .clone()
                .with_center(center)
                .with_parent_translation(parent_tf.translation())
                .with_merge_distance(**merge_distance)
                .check()
                .then_some(children)
        })
        .flatten()
    {
        cmd.entity(*child)
            .queue_silenced(|mut entity: EntityWorldMut| -> Result {
                entity.insert(Merging);
                Ok(())
            });
    }
}

pub fn merge(
    q_chunk: Query<(Entity, &ChildOf), (With<Merging>, With<Chunk>)>,
    mut mw_merge: MessageWriter<MergeChunks>,
) {
    mw_merge.write_batch(
        get_parent_child_map(q_chunk)
            .into_iter()
            .map(|data| data.into()),
    );
}

pub fn handle_merge(mut cmd: Commands, mut mr_merge: MessageReader<MergeChunks>) {
    for MergeChunks { parent, children } in mr_merge.read() {
        #[cfg(feature = "trace")]
        debug!("Merging Chunks: {children:?} into {parent}");

        despawn(&mut cmd, children.clone());

        cmd.entity(*parent).insert(CanSplit);
    }
}

fn get_parent_child_map<T: QueryFilter>(
    q_chunk: Query<(Entity, &ChildOf), T>,
) -> HashMap<Entity, Vec<Entity>> {
    q_chunk
        .iter()
        .fold(HashMap::new(), |mut map, (entity, parent)| {
            map.entry(parent.get()).or_default().push(entity);
            map
        })
}
