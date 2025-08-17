use crate::prelude::*;
use bevy::prelude::*;

pub fn check_unprocessed_items(
    mut cmd: Commands,
    q_unprocessed_children: Query<Entity, Without<ScatterLayerChildProcessed>>,
    q_layer: Query<(Entity, &Children), (Without<ScatterLayerProcessed>, With<ScatterLayer>)>,
) {
    for (layer, children) in &q_layer {
        let mut unprocessed_item_count = 0;
        for item in children {
            if q_unprocessed_children.get(*item).is_err() {
                continue;
            };

            unprocessed_item_count += 1
        }

        if unprocessed_item_count == 0 {
            debug!("ScatterLayer {layer} processed.");

            cmd.entity(layer).insert(ScatterLayerProcessed);
        }
    }
}

pub fn check_unprocessed_layers(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_unprocessed_layers: Query<&Children, (Without<ScatterLayerProcessed>, With<ScatterLayer>)>,
) {
    for (root, children) in &q_roots {
        debug!("Collecting ScatterAssets in root {:?}...", root);

        let mut unprocessed_layer_count = 0;

        for layer in children.iter() {
            if q_unprocessed_layers.get(layer).is_err() {
                continue;
            };

            unprocessed_layer_count += 1;
        }

        if unprocessed_layer_count == 0 {
            debug!("ScatterRoot {root} is ready.");

            cmd.entity(root).insert(ScatterRootProcessed);
        }
    }
}
