pub mod plugin;
pub mod material;
pub mod extension;

pub use plugin::ExtendedWindAffectedPlugin;

pub mod prelude {
    pub use super::plugin::ExtendedWindAffectedPlugin;

    pub use super::material::*;
    pub use super::extension::*;
}