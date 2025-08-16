use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter_root<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    mut trigger: On<Scatter<TIn, TOut>>,
    mut cmd: Commands,
    q_root: Query<&ScatterRoot>,
) {
    trigger.propagate(false);

    let root = trigger.target();

    let Ok(layers) = q_root.get(root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    debug!("Scattering root: {root}");

    cmd.trigger_targets(
        Scatter::<TIn, TOut>::new(),
        layers.iter().collect::<Vec<_>>(),
    );
}
