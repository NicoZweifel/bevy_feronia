use crate::asset::backend::components::AssetItem;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemId;

/// The current backend registered through [`ScatterApp::set_scatter_asset_backend`]
#[derive(Resource, Debug, Clone, DerefMut, Deref)]
pub struct ScatterAssetBackend(pub SystemId<In<()>, Result<Vec<AssetItem>>>);
