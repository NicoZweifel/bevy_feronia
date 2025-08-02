use crate::prelude::*;
use crate::scatter::utils::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::reflect::List;
use bevy::render::primitives::Aabb;

pub fn setup_root(
    mut cmd: Commands,
    q_root: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    q_children: Query<&Children>,
    q_aabb: Query<&Aabb>,
) {
    for root_entity in &q_root {
        let mut root_aabb: Option<Aabb> = None;

        for descendant_entity in q_children.iter_descendants(root_entity) {
            let Ok(descendant_aabb) = q_aabb.get(descendant_entity) else {
                continue;
            };

            match root_aabb.as_mut() {
                Some(existing_aabb) => {
                    combine_aabbs(existing_aabb, descendant_aabb);
                }
                None => {
                    root_aabb = Some(descendant_aabb.clone());
                }
            }
        }

        if let Some(aabb) = root_aabb {
            cmd.entity(root_entity).insert(aabb);
        }
    }
}
