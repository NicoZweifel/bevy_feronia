use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use crate::scatter::events::ScatterResults;
use bevy_ecs::prelude::*;

#[cfg(feature = "trace")]
use tracing::{debug, warn};

pub fn scatter_observer<T>(
    trigger: On<ScatterResults<T>>,
    q_layer: Query<&ScatterLayer, With<ScatterLayerType<T>>>,
    q_items: Query<&ScatterItemAsset<T>, With<ScatterItem>>,
    mut queue: ResMut<SpawnScatterAssetsEventQueue<T>>,
) where
    T: ScatterMaterial,
{
    let layer = trigger.layer;
    let Ok(scatter_items) = q_layer.get(layer) else {
        #[cfg(feature = "trace")]
        debug!("ScatterLayer {layer} not found!");
        return;
    };

    #[cfg(feature = "trace")]
    debug!("ScatterObserver triggered! Queueing Spawn Messages for layer {layer}...");

    (**queue).push_back(
        SpawnScatterAssets::<T>::from(trigger).with_items(
            scatter_items
                .iter()
                .filter_map(|e| q_items.get(e).ok().cloned()),
        ),
    );
}
