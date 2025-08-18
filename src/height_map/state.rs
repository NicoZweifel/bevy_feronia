use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum HeightMapState {
    #[default]
    Loading,
    Setup,
    Ghost,
    Baking,
    Saving,
    Ready,
}
