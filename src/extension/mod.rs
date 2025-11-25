pub mod components;
pub mod material;
pub mod material_extension;
pub mod plugin;
pub mod scatter;

pub use plugin::ExtendedWindAffectedPlugin;

pub use scatter::scatter_layer;

pub mod prelude {
    pub use super::components::*;
    pub use super::material::*;
    pub use super::material_extension::*;
    pub use super::plugin::*;
}
