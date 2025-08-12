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
    let Ok(scatter_items) = q_layer.get(trigger.layer) else {
        warn!("No ScatterLayer found!");
        return;
    };

    let items = scatter_items
        .iter()
        .filter_map(|x| q_items.get(x).ok().map(|x| x.clone()))
        .collect::<Vec<_>>();

    let trigger = SpawnTrigger::from(trigger);
    let event = SpawnProtoTypes::from(trigger).with_items(items);

    ew_spawn.write(event);
}
