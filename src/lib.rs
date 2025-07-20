pub mod plugin;
pub mod systems;
pub mod components;
pub mod instancing;
pub mod extension;
pub mod core;
pub mod resources;

pub use crate::plugin::{WindMaterialPlugin, WindPlugin};

pub mod prelude {

    pub use crate::plugin::{WindMaterialPlugin, WindPlugin};

    pub use crate::resources::*;
    pub use crate::components::*;
    pub use crate::core::*;

    pub use crate::instancing::prelude::*;
    pub use crate::extension::prelude::*;
}