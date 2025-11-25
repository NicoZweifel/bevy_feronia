use bevy_state::prelude::States;

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ScatterState {
    #[default]
    Loading,
    Setup,
    Collecting,
    Ready,
}
