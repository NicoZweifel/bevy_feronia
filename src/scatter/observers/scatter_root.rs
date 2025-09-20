use crate::prelude::*;
use bevy::prelude::*;

pub fn scatter_root<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    trigger: On<Scatter<TIn, TOut>>,
    mut cmd: Commands,
    q_root: Query<&ScatterRoot>,
) {
    let root = trigger.entity;

    let Ok(layers) = q_root.get(root) else {
        warn!("ScatterRoot not found!");
        return;
    };

    debug!("Scattering root: {root}");

    layers
        .iter()
        .map(|x| Scatter::<TIn, TOut>::new(x))
        .for_each(|x| cmd.trigger(x));
}
