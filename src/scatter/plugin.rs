use crate::scatter::components::{ScatterLayer, ScatterRoot};
use crate::scatter::events::{Scatter, ScatterResults};
use crate::scatter::systems::setup;
use bevy::app::*;

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Scatter<ScatterRoot>>()
            .add_event::<Scatter<ScatterLayer>>()
            .add_event::<ScatterResults>()
            .add_systems(Update, (setup::setup_root, setup::setup_layer));
    }
}
