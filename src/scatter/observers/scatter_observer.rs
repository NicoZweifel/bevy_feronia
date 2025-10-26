use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::prelude::*;

pub fn scatter_observer<TOut, TIn>(
    trigger: On<ScatterResults<TOut, TIn>>,
    q_layer: Query<&ScatterLayer, With<ScatterLayerType<TOut, TIn>>>,
    q_items: Query<&ScatterItemAsset<TOut>, With<ScatterItem>>,
    mut mw_spawn: MessageWriter<SpawnProtoTypes<TOut>>,
) where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    let layer = trigger.layer;

    let Ok(scatter_items) = q_layer.get(layer) else {
        warn!("ScatterLayer {layer} not found!");
        return;
    };

    let items = scatter_items
        .iter()
        .filter_map(|x| q_items.get(x).ok().cloned());

    let trigger = SpawnTrigger::from(trigger);

    let event = SpawnProtoTypes::from(trigger).with_items(items.collect());

    debug!("ScatterObserver triggered! Writing Spawn Events...");

    mw_spawn.write(event);
}
