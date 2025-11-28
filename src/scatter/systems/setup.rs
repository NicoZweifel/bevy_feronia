use crate::prelude::*;
use crate::scatter::utils::*;
use bevy_camera::primitives::Aabb;
use bevy_ecs::prelude::*;
use bevy_state::state::NextState;

#[cfg(feature = "trace")]
use tracing::debug;

pub fn transition_to_collecting(
    q_pending_roots: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    mut next_state: ResMut<NextState<ScatterState>>,
) {
    if !q_pending_roots.is_empty() {
        return;
    };

    #[cfg(feature = "trace")]
    debug!("Setting ScatterState::Collecting.");

    next_state.set(ScatterState::Collecting);
}

pub fn setup_root_aabb(
    mut cmd: Commands,
    q_root: Query<(Entity, &Children), (With<ScatterRoot>, Without<Aabb>)>,
    q_children: Query<&Children, Without<ScatterLayer>>,
    q_aabb: Query<&Aabb>,
) {
    for (root_entity, children) in &q_root {
        let aabb: Option<Aabb> = children
            .iter()
            .flat_map(|child| q_children.iter_descendants(child))
            .filter_map(|entity| q_aabb.get(entity).ok())
            .fold(None, |aabb, child| {
                aabb.map(|aabb| combine_aabbs(&aabb, child))
                    .or(Some(*child))
            });

        #[cfg(feature = "trace")]
        debug!("Calculated and inserted AABB for ScatterRoot. {aabb:?}");

        if let Some(aabb) = aabb {
            cmd.entity(root_entity).insert(aabb);
        }
    }
}
