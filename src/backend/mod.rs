use crate::prelude::AssetItem;
use crate::prelude::ScatterAssetBackend;

use bevy_app::App;
use bevy_ecs::prelude::*;

/// Used to implement [`ScatterApp::set_asset_source_backend`] on [`App`]
pub trait ScatterApp {
    /// Set the backend for collecting [`AssetItem`]s, from which [`ScatterAssetCreationRequest`]s will be created.
    ///
    /// Only one backend can be set at a time for each source Material `T`.
    /// Setting a backend will replace any existing backend for the same Material type `T`. By default, no backend is set.
    ///
    /// The backend is supposed to return [`AssetItem`], which describes the relations of a discovered part.
    fn set_scatter_asset_backend<M>(
        &mut self,
        system: impl IntoSystem<In<()>, Result<Vec<AssetItem>>, M> + 'static,
    ) -> &mut App;
}

impl ScatterApp for App {
    fn set_scatter_asset_backend<M>(
        &mut self,
        system: impl IntoSystem<In<()>, Result<Vec<AssetItem>>, M> + 'static,
    ) -> &mut App {
        let id = self.register_system(system);
        self.world_mut().insert_resource(ScatterAssetBackend(id));
        self
    }
}
