use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::Commands;
use bevy_state::state::NextState;
#[cfg(feature = "tracing")]
use tracing::debug;

pub fn check_unprocessed_items(
    mut cmd: Commands,
    q_layer: Query<
        (Entity, &Children, Option<&Name>),
        (Without<ScatterLayerProcessed>, With<ScatterLayer>),
    >,
    q_unprocessed: Query<Entity, Without<ScatterLayerChildProcessed>>,
) {
    for (layer, children, _name) in &q_layer {
        if children
            .iter()
            .map(|e| q_unprocessed.get(e))
            .any(|r| r.is_ok())
        {
            #[cfg(feature = "tracing")]
            debug!(
                "ScatterLayer {} {layer} has unprocessed children.",
                _name.cloned().unwrap_or_default()
            );
            continue;
        }

        if children.len() > 0 {
            #[cfg(feature = "tracing")]
            debug!(
                "ScatterLayer {} {layer} processed.",
                _name.cloned().unwrap_or_default()
            );

            cmd.entity(layer).insert(ScatterLayerProcessed);
        };
    }
}

pub fn check_unprocessed_layers(
    mut cmd: Commands,
    q_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    q_unprocessed_layers: Query<&Children, (Without<ScatterLayerProcessed>, With<ScatterLayer>)>,
) {
    for (root, children) in &q_roots {
        #[cfg(feature = "tracing")]
        debug!("Collecting ScatterAssets in root {:?}...", root);

        if children
            .iter()
            .map(|e| q_unprocessed_layers.get(e))
            .any(|r| r.is_ok())
        {
            #[cfg(feature = "tracing")]
            debug!("ScatterRoot {root} has unprocessed layers.");
            continue;
        }

        if children.len() > 0 {
            #[cfg(feature = "tracing")]
            debug!("ScatterRoot {root} is processed.");

            cmd.entity(root).insert(ScatterRootProcessed);
        }
    }
}

pub fn check_unprocessed_root(
    q_unprocessed_roots: Query<(Entity, &ScatterRoot), Without<ScatterRootProcessed>>,
    mut ns_scatter: ResMut<NextState<ScatterState>>,
) {
    if !q_unprocessed_roots.is_empty() {
        return;
    };

    #[cfg(feature = "tracing")]
    debug!("Setting ScatterState::Ready.");
    ns_scatter.set(ScatterState::Ready);
}
