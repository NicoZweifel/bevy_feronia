use crate::prelude::*;
use bevy::prelude::*;

pub fn clear_scatter_roots(
    mut cmd: Commands,
    mut mr_clear_root: MessageReader<ClearScatterRoot>,
    mut mw_clear_layers: MessageWriter<ClearScatterLayer>,
    q_root: Query<(Entity, &ScatterRoot)>,
) {
    for root in mr_clear_root.read() {
        let Ok((root, layers)) = q_root.get(**root) else {
            continue;
        };

        cmd.entity(root).insert(ScatterOccupancyMap::default());

        mw_clear_layers.write_batch(layers.iter().map(|e| e.into()));
    }
}
