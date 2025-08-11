use crate::prelude::*;
use crate::scatter::observers::scatter_observer;
use bevy::prelude::*;

pub fn extended_scatter_observer(
    trigger: On<ScatterResults>,
    q_layer: Query<&ScatterLayer>,
    q_items: Query<&ScatterItemAsset<ExtendedWindAffectedMaterial>, With<ScatterItem>>,
    ew_spawn: EventWriter<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
) {
    scatter_observer::<
        ScatterAsset<ExtendedWindAffectedMaterial>,
        StandardMaterial,
        ExtendedWindAffectedMaterial,
    >(trigger, q_layer, q_items, ew_spawn);
}
