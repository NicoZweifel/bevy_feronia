use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum HeightMapState {
    #[default]
    Setup,
    Ghost,
    Baking,
    Saving,
    Ready,
}
