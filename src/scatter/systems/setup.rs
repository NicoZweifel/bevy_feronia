use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::reflect::List;
use bevy::render::primitives::Aabb;

pub fn transition_to_ready_state(
    q_pending_roots: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    mut next_state: ResMut<NextState<ScatterState>>,
) {
    if !q_pending_roots.is_empty() {
        return;
    };

    next_state.set(ScatterState::Ready);
}

pub fn setup_root_aabb(
    mut cmd: Commands,
    q_root: Query<(Entity, &Children), (With<ScatterRoot>, Without<Aabb>)>,
    q_children: Query<&Children, Without<ScatterLayer>>,
    q_aabb: Query<&Aabb>,
) {
    for (root_entity, children) in &q_root {
        let mut root_aabb: Option<Aabb> = None;

        for child in children.iter() {
            for descendant_entity in q_children.iter_descendants(child) {
                let Ok(descendant_aabb) = q_aabb.get(descendant_entity) else {
                    continue;
                };

                if let Some(existing_aabb) = &mut root_aabb {
                    *existing_aabb = combine_aabbs(existing_aabb, descendant_aabb);
                } else {
                    root_aabb = Some(*descendant_aabb);
                }
            }
        }

        let Some(aabb) = root_aabb else {
            continue;
        };

        debug!("Calculated and inserted AABB for ScatterRoot. {aabb:?}");
        cmd.entity(root_entity).insert(aabb);
    }
}
