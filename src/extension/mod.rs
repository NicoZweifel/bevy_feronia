pub mod extension;
pub mod material;
pub mod observers;
pub mod plugin;
pub mod scatter;
pub mod spawn;

pub use plugin::ExtendedWindAffectedPlugin;

pub mod prelude {
    pub use super::extension::*;
    pub use super::material::*;
    pub use super::plugin::*;
}
