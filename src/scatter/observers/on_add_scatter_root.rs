use crate::prelude::*;
use crate::scatter::observers::scatter_root;
use bevy::prelude::*;

pub fn on_add_scatter_root<
    TIn: Material,
    TOut: WindAffectable<ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
>(
    trigger: On<Add, ScatterRoot>,
    mut cmd: Commands,
) {
    cmd.entity(trigger.target())
        .observe(scatter_root::<TIn, TOut>);
}
