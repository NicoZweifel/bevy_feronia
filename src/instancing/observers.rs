use crate::prelude::*;
use crate::scatter::observers::scatter_observer;
use bevy::prelude::*;

pub fn instanced_scatter_observer(
    trigger: On<ScatterResults>,
    q_layer: Query<&ScatterLayer>,
    q_items: Query<&ScatterItemAsset<InstancedWindAffectedMaterial>, With<ScatterItem>>,
    ew_spawn: EventWriter<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
) {
    scatter_observer::<
        ScatterAsset<InstancedWindAffectedMaterial>,
        StandardMaterial,
        InstancedWindAffectedMaterial,
    >(trigger, q_layer, q_items, ew_spawn);
}
