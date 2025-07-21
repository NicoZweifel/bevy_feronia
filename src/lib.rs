pub mod components;
pub mod core;
pub mod extension;
pub mod instancing;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use crate::plugin::{WindMaterialPlugin, WindPlugin};

pub mod prelude {

    pub use crate::plugin::{WindMaterialPlugin, WindPlugin};

    pub use crate::components::*;
    pub use crate::core::*;
    pub use crate::resources::*;

    pub use crate::extension::prelude::*;
    pub use crate::instancing::prelude::*;
}
