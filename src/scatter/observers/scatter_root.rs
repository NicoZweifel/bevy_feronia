use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use bevy_utils::default;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub fn scatter_root<T>(
    trigger: On<Scatter<T>>,
    mut cmd: Commands,
    q_root: Query<(&ScatterRoot, &HierarchicalScatterState<T>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<T>>)>,
) where
    T: ScatterMaterial,
{
    let root_entity = trigger.entity;

    let Ok((layers, state)) = q_root.get(root_entity) else {
        return;
    };

    if state.pending_tasks > 0 {
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
    q_requests: Query<&ScatterRequest<T>>,
) where
    T: ScatterMaterial,
{
    let layer = trigger.layer;

    let Ok(parent) = q_layer_parent.get(layer) else {
        return;
    };

    let root_entity = parent.get();

    let Ok((mut state, mut map)) = q_roots.get_mut(root_entity) else {
        return;
    };

    if let Ok(avoidance) = q_avoidance.get(layer) {
        let base_radius = avoidance;
        let container_transform = trigger.container_global_transform;

        for instance in &trigger.data {
            let world_pos = container_transform.transform_point(instance.transform.translation);
            let max_scale = instance.transform.scale.max_element();

            map.add_sphere(world_pos, **base_radius * max_scale);
        }
    }

    if state.pending_tasks > 0 {
        state.pending_tasks -= 1;
    }

    if state.pending_tasks > 0 {
        #[cfg(feature = "trace")]
        debug!("Layer {layer} has pending tasks...");
        return;
    }

    let has_pending_requests = q_requests.iter().any(|req| req.layer_entity == layer);

    if has_pending_requests {
        #[cfg(feature = "trace")]
        debug!("Layer {layer} has pending requests...",);
        return;
    }

    state.current_layer_index += 1;

    if state.current_layer_index < state.ordered_layers.len() {
        let next = state.ordered_layers[state.current_layer_index];
        #[cfg(feature = "trace")]
        debug!("Hierarchical scatter advancing to layer: {:?}", next);
        cmd.trigger(Scatter::<T>::new(next));
        return;
    }

    cmd.entity(root_entity)
        .insert(HierarchicalScatterState::<T>::default());

    cmd.trigger(ScatterFinished::<T>::new(root_entity));

    #[cfg(feature = "trace")]
    debug!("Hierarchical scatter finished for root: {:?}", root_entity);
}
