use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy::prelude::*;

pub fn scatter_observer<TTypes, TType, TIn, TOut>(
    trigger: On<ScatterResults>,
    q_layer: Query<&ScatterLayer>,
    q_items: Query<&ScatterItemType<TOut>, With<ScatterItem>>,
    mut ew_spawn: EventWriter<SpawnProtoTypes<TOut>>,
) where
    TTypes: Resource + ProtoTypes<TOut, TType>,
    TType: ProtoType<TOut> + Asset + Clone,
    TIn: Material,
    TOut: Asset + Clone,
{
    let Ok(scatter_items) = q_layer.get(trigger.layer) else {
        warn!("No ScatterLayer found!");
        return;
    };

    ew_spawn.write(SpawnProtoTypes::new(
        scatter_items
            .iter()
            .filter_map(|x| q_items.get(x).ok().map(|x| x.clone()))
            .collect::<Vec<_>>(),
        SpawnTrigger::from(trigger),
    ));
}
