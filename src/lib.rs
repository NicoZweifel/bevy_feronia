pub mod chunking;
pub mod components;
pub mod core;
pub mod density_map;
pub mod extension;
pub mod height_map;
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

    pub use crate::chunking::prelude::*;
    pub use crate::density_map::*;
    pub use crate::extension::prelude::*;
    pub use crate::height_map::prelude::*;
    pub use crate::instancing::prelude::*;
}
