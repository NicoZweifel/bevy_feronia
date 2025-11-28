use bevy_state::prelude::States;

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
