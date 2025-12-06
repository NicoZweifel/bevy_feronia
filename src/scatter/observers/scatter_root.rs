use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use bevy_utils::default;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub fn scatter_root<T>(
    trigger: On<Scatter<T>>,
    mut cmd: Commands,
    q_root: Query<(&ScatterRoot, Option<&HierarchicalScatterState<T>>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<T>>)>,
) where
    T: ScatterMaterial,
{
    let root_entity = trigger.entity;

    let Ok((layers, state)) = q_root.get(root_entity) else {
        return;
    };

    if state.is_some() {
        #[cfg(feature = "trace")]
        warn!(
            "Hierarchical scatter is already in progress for root {:?}. Ignoring new request.",
            root_entity
        );
        return;
    }

    let ordered_layers: Vec<Entity> = layers.iter().filter_map(|e| q_layer.get(e).ok()).collect();
    if ordered_layers.is_empty() {
        return;
    }

    #[cfg(feature = "trace")]
    debug!(
        "Clearing previous scatter results for root: {:?}",
        root_entity
    );

    #[cfg(feature = "trace")]
    debug!(
        "Starting hierarchical scatter on root: {:?}. Resetting occupancy map.",
        root_entity
    );

    cmd.entity(root_entity)
        .insert((HierarchicalScatterState::<T> {
            ordered_layers: ordered_layers.clone(),
            current_layer_index: 0,
            ..default()
        },));

    let first_layer_entity = ordered_layers[0];

    cmd.trigger(Scatter::<T>::new(first_layer_entity));
}

pub fn hierarchical_scatter<T>(
    trigger: On<ScatterResults<T>>,
    mut cmd: Commands,
    q_layer_parent: Query<&ScatterLayerOf, With<ScatterLayer>>,
    mut q_roots: Query<(&mut HierarchicalScatterState<T>, &mut ScatterOccupancyMap)>,
    q_avoidance: Query<&Avoidance>,
) where
    T: ScatterMaterial,
{
    let finished_layer = trigger.layer;

    let Ok(parent) = q_layer_parent.get(finished_layer) else {
        return;
    };

    let root_entity = parent.get();

    let Ok((mut state, mut map)) = q_roots.get_mut(root_entity) else {
        return;
    };

    if let Ok(avoidance) = q_avoidance.get(finished_layer) {
        let base_radius = avoidance;
        let container_transform = trigger.container_transform;

        for instance in &trigger.data {
            let world_pos = container_transform.transform_point(instance.transform.translation);
            let max_scale = instance.transform.scale.max_element();

            map.add_circle(world_pos, **base_radius * max_scale);
        }
    }

    state.current_layer_index += 1;

    if state.current_layer_index < state.ordered_layers.len() {
        let next_layer_entity = state.ordered_layers[state.current_layer_index];
        #[cfg(feature = "trace")]
        debug!(
            "Hierarchical scatter advancing to layer: {:?}",
            next_layer_entity
        );
        cmd.trigger(Scatter::<T>::new(next_layer_entity));
    } else {
        #[cfg(feature = "trace")]
        debug!("Hierarchical scatter finished for root: {:?}", root_entity);
        cmd.entity(root_entity)
            .remove::<HierarchicalScatterState<T>>();
    }
}
