use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy_ecs::prelude::*;

#[cfg(feature = "tracing")]
use tracing::{debug, warn};

pub fn scatter_observer<T>(
    trigger: On<ScatterResults<T>>,
    q_layer: Query<&ScatterLayer, With<ScatterLayerType<T>>>,
    q_items: Query<&ScatterItemAsset<T>, With<ScatterItem>>,
    mut mw_spawn: MessageWriter<SpawnScatterAssets<T>>,
) where
    T: ScatterMaterial,
{
    let layer = trigger.layer;
    let Ok(scatter_items) = q_layer.get(layer) else {
        #[cfg(feature = "tracing")]
        warn!("ScatterLayer {layer} not found!");
        return;
    };

    #[cfg(feature = "tracing")]
    debug!("ScatterObserver triggered! Writing Spawn Messages for layer {layer}...");

    mw_spawn.write(
        SpawnScatterAssets::<T>::from(trigger).with_items(
            scatter_items
                .iter()
                .filter_map(|e| q_items.get(e).ok().cloned()),
        ),
    );
}
