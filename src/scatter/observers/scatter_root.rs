use crate::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

pub fn scatter_root<TOut, TIn>(
    trigger: On<Scatter<TOut, TIn>>,
    mut cmd: Commands,
    q_root: Query<(&ScatterRoot, Option<&HierarchicalScatterState<TOut, TIn>>)>,
    q_layer: Query<Entity, (With<ScatterLayer>, With<ScatterLayerType<TOut, TIn>>)>,
) where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    let root_entity = trigger.entity;

    let Ok((layers, state)) = q_root.get(root_entity) else {
        return;
    };

    if state.is_some() {
        warn!(
            "Hierarchical scatter is already in progress for root {:?}. Ignoring new request.",
            root_entity
        );
        return;
    }

    let ordered_layers: Vec<Entity> = layers.iter().filter_map(|x| q_layer.get(x).ok()).collect();
    if ordered_layers.is_empty() {
        return;
    }

    debug!(
        "Clearing previous scatter results for root: {:?}",
        root_entity
    );

    debug!(
        "Starting hierarchical scatter on root: {:?}. Resetting occupancy map.",
        root_entity
    );

    cmd.entity(root_entity)
        .insert((HierarchicalScatterState::<TOut, TIn> {
            ordered_layers: ordered_layers.clone(),
            current_layer_index: 0,
            ..default()
        },));

    let first_layer_entity = ordered_layers[0];
    cmd.trigger(Scatter::<TOut, TIn>::new(first_layer_entity));
}

pub fn hierarchical_scatter<TOut, TIn>(
    trigger: On<ScatterResults<TOut, TIn>>,
    mut cmd: Commands,
    q_layer_parent: Query<&ScatterLayerOf, With<ScatterLayer>>,
    mut q_roots: Query<(
        &mut HierarchicalScatterState<TOut, TIn>,
        &mut ScatterOccupancyMap,
    )>,
    q_avoidance: Query<&Avoidance>,
) where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
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
        let radius_sq = avoidance.powi(2);
        let container_transform = trigger.container_transform;

        for instance in &trigger.data {
            let world_pos = container_transform.transform_point(instance.transform.translation);
            map.occupied_zones.push(AvoidanceData {
                world_pos,
                radius_sq,
            });
        }
    }

    state.current_layer_index += 1;

    if state.current_layer_index < state.ordered_layers.len() {
        let next_layer_entity = state.ordered_layers[state.current_layer_index];
        debug!(
            "Hierarchical scatter advancing to layer: {:?}",
            next_layer_entity
        );
        cmd.trigger(Scatter::<TOut, TIn>::new(next_layer_entity));
    } else {
        debug!("Hierarchical scatter finished for root: {:?}", root_entity);
        cmd.entity(root_entity)
            .remove::<HierarchicalScatterState<TOut, TIn>>();
    }
}
