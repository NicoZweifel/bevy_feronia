pub mod assets;
pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

pub mod prelude {
    pub use super::assets::*;
    pub use super::components::*;
    pub use super::plugin::*;
}
