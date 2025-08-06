use crate::prelude::*;
use crate::scatter::observers::scatter_observer;
use bevy::prelude::*;

pub fn instanced_scatter_observer(
    trigger: On<ScatterResults>,
    q_layer: Query<&ScatterLayer>,
    q_items: Query<&ScatterItemType<InstancedWindAffectedMaterial>, With<ScatterItem>>,
    ew_spawn: EventWriter<SpawnProtoTypes<InstancedWindAffectedMaterial>>,
) {
    scatter_observer::<
        ScatterAssets<InstancedWindAffectedMaterial>,
        ScatterAsset<InstancedWindAffectedMaterial>,
        StandardMaterial,
        InstancedWindAffectedMaterial,
    >(trigger, q_layer, q_items, ew_spawn);
}
