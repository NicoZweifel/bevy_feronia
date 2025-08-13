use crate::prelude::*;
use bevy::prelude::*;

pub fn check_unprocessed_layers(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_unprocessed: Query<&Children, (Without<ScatterLayerProcessed>,)>,
) {
    for (root, children) in &q_roots {
        debug!("Collecting ScatterAssets in root {:?}...", root);

        let mut unprocessed_layer_count = 0;

        for layer in children.iter() {
            if q_unprocessed.get(layer).is_err() {
                continue;
            };

            unprocessed_layer_count += 1;
        }

        if unprocessed_layer_count == 0 {
            debug!(
                "No unprocessed ScatterLayers found in root {:?}... ScatterRoot is ready.",
                root
            );
            cmd.entity(root).insert(ScatterRootProcessed);
        }
    }
}
