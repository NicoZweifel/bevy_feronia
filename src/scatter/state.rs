use bevy_ecs::prelude::SystemSet;
use bevy_state::prelude::States;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScatterSet {
    Loading,
    Setup,
    Collecting,
    Ready,
    Clean,
}

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ScatterState {
    #[default]
    Loading,
    Setup,
    Collecting,
    Ready,
}
