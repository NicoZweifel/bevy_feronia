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
    let Ok(layers) = q_root.get(trigger.target()) else {
        warn!("ScatterRoot not found!");
        return;
    };

    info!("Scattering root: {:?}", trigger.target());

    trigger.propagate(false);

    cmd.trigger_targets(
        Scatter::<TIn, TOut>::new(),
        layers.iter().collect::<Vec<_>>(),
    );
}
