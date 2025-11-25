pub mod material;
pub mod material_extension;
pub mod plugin;
pub mod scatter;
pub mod components;

pub use plugin::ExtendedWindAffectedPlugin;

pub use scatter::scatter_layer;

pub mod prelude {
    pub use super::material::*;
    pub use super::material_extension::*;
    pub use super::plugin::*;
    pub use super::components::*;
}
