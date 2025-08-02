use crate::prelude::*;
use crate::scatter::observers::*;
use crate::scatter::utils::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

pub fn setup_root(
    mut cmd: Commands,
    root_query: Query<Entity, (With<ScatterRoot>, Without<Aabb>)>,
    children_query: Query<&Children>,
    aabb_query: Query<&Aabb>,
) {
    for root_entity in &root_query {
        let mut root_aabb: Option<Aabb> = None;

        for descendant_entity in children_query.iter_descendants(root_entity) {
            if let Ok(descendant_aabb) = aabb_query.get(descendant_entity) {
                match root_aabb.as_mut() {
                    Some(existing_aabb) => {
                        combine_aabbs(existing_aabb, descendant_aabb);
                    }
                    None => {
                        root_aabb = Some(descendant_aabb.clone());
                    }
                }
            }
        }

        if let Some(aabb) = root_aabb {
            cmd.entity(root_entity).insert(aabb);
            cmd.entity(root_entity)
                .observe(generate_scatter_points_root);
        }
    }
}

pub fn setup_layer(
    mut cmd: Commands,
    layer_query: Query<(Entity, &ChildOf), (With<ScatterLayer>, Without<ScatterLayerOf>)>,
    root_query: Query<Option<&ChunkRoot>, With<ScatterRoot>>,
) {
    for (layer, root_layer) in &layer_query {
        cmd.entity(layer).insert(ScatterLayerOf(root_layer.get()));

        let chunk_root = root_query.get(root_layer.get()).unwrap();
        if chunk_root.is_some() {
            cmd.entity(layer)
                .observe(generate_scatter_points_layer_chunked);
        } else {
            cmd.entity(layer).observe(generate_scatter_points_layer);
        }
    }
}
