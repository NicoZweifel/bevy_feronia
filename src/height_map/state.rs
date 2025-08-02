use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum HeightMapState {
    #[default]
    Loading,
    Baking,
    Saving,
    Ready,
}
