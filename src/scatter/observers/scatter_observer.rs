use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::prelude::*;

pub fn scatter_observer<TIn, TOut>(
    trigger: On<ScatterResults<TIn, TOut>>,
    q_layer: Query<&ScatterLayer, With<ScatterLayerType<TIn, TOut>>>,
    q_items: Query<&ScatterItemAsset<TOut>, With<ScatterItem>>,
    mut ew_spawn: EventWriter<SpawnProtoTypes<TOut>>,
) where
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    let layer = trigger.layer;

    let Ok(scatter_items) = q_layer.get(layer) else {
        warn!("ScatterLayer {layer} not found!");
        return;
    };

    let items = scatter_items
        .iter()
        .filter_map(|x| q_items.get(x).ok().map(|x| x.clone()));

    let trigger = SpawnTrigger::from(trigger);

    let event = SpawnProtoTypes::from(trigger).with_items(items.collect());

    ew_spawn.write(event);
}
