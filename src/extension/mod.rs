pub mod extension;
pub mod material;
pub mod plugin;

pub use plugin::ExtendedWindAffectedPlugin;

pub mod prelude {
    pub use super::plugin::ExtendedWindAffectedPlugin;

    pub use super::extension::*;
    pub use super::material::*;
}
