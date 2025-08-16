use crate::prelude::*;
use crate::scatter::observers::scatter_observer;
use bevy::prelude::*;

pub fn instanced_scatter_observer(
    trigger: On<ScatterResults<StandardMaterial, InstancedWindAffectedMaterial>>,
    q_layer: Query<
        &ScatterLayer,
        With<ScatterLayerType<StandardMaterial, InstancedWindAffectedMaterial>>,
    >,
    q_items: Query<&ScatterItemAsset<InstancedWindAffectedMaterial>, With<ScatterItem>>,
    ew_spawn: EventWriter<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
) {
    scatter_observer::<StandardMaterial, InstancedWindAffectedMaterial>(
        trigger, q_layer, q_items, ew_spawn,
    );
}
