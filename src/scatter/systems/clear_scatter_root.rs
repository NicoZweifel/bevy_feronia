use crate::prelude::*;
use bevy_ecs::prelude::*;

#[cfg(feature = "trace")]
use tracing::warn;

pub fn clear_scatter_roots(
    mut cmd: Commands,
    mut mr_clear_root: MessageReader<ClearScatterRoot>,
    mut mw_clear_layers: MessageWriter<ClearScatterLayer>,
    q_root: Query<(Entity, &ScatterRoot)>,
) {
    let layers = mr_clear_root
        .read()
        .filter_map(|root| {
            q_root
                .get(**root)
                .inspect_err(|_| {
                    #[cfg(feature = "trace")]
                    warn!(
                        "No `ScatterRoot` found for root entity {:?}. Skipping.",
                        root
                    );
                })
                .map(|(root, layers)| {
                    cmd.entity(root).insert(ScatterOccupancyMap::default());
                    layers.iter()
                })
                .ok()
        })
        .flatten();

    mw_clear_layers.write_batch(layers.map(ClearScatterLayer));
}
