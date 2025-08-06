use bevy::prelude::States;

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ScatterState {
    #[default]
    Setup,
    Ready,
}
