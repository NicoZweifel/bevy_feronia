use crate::prelude::*;
use crate::scatter::observers::scatter_observer;
use bevy::prelude::*;

pub fn extended_scatter_observer(
    trigger: On<ScatterResults<ExtendedWindAffectedMaterial>>,
    q_layer: Query<&ScatterLayer, With<ScatterLayerType<ExtendedWindAffectedMaterial>>>,
    q_items: Query<&ScatterItemAsset<ExtendedWindAffectedMaterial>, With<ScatterItem>>,
    mw_spawn: MessageWriter<SpawnProtoTypes<ExtendedWindAffectedMaterial>>,
) {
    scatter_observer::<ExtendedWindAffectedMaterial, StandardMaterial>(
        trigger, q_layer, q_items, mw_spawn,
    );
}
