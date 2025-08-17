use crate::prelude::*;
use bevy::prelude::*;

pub fn check_unprocessed_items(
    mut cmd: Commands,
    q_layer: Query<(Entity, &Children), (Without<ScatterLayerProcessed>, With<ScatterLayer>)>,
    q_unprocessed_children: Query<Entity, Without<ScatterLayerChildProcessed>>,
) {
    for (layer, children) in &q_layer {
        if children
            .iter()
            .map(|e| q_unprocessed_children.get(e))
            .any(|r| r.is_ok())
        {
            debug!("ScatterLayer {layer} has unprocessed children.");
            continue;
        }

        debug!("ScatterLayer {layer} processed.");

        cmd.entity(layer).insert(ScatterLayerProcessed);
    }
}

pub fn check_unprocessed_layers(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_unprocessed_layers: Query<&Children, (Without<ScatterLayerProcessed>, With<ScatterLayer>)>,
) {
    for (root, children) in &q_roots {
        debug!("Collecting ScatterAssets in root {:?}...", root);

        if children
            .iter()
            .map(|e| q_unprocessed_layers.get(e))
            .any(|r| r.is_ok())
        {
            debug!("ScatterRoot {root} has unprocessed layers.");
            continue;
        }

        debug!("ScatterRoot {root} is ready.");

        cmd.entity(root).insert(ScatterRootProcessed);
    }
}
