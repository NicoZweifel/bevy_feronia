pub mod components;
pub mod draw;
pub mod material;
pub mod node;
pub mod pipeline;
pub mod plugin;
pub mod prepare;
mod resources;
pub mod scatter;
pub mod systems;

pub use plugin::*;

pub use scatter::scatter_layer;

pub mod prelude {
    pub use super::plugin::*;
    pub use super::{components::*, material::*};
}
