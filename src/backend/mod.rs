use crate::prelude::ScatterAssetBackend;
use crate::prelude::{AssetPart, ScatterSet, ScatterState};

use bevy_app::{App, PostUpdate, PreUpdate, Update};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_state::condition::in_state;

/// Used to implement [`ScatterApp::set_asset_source_backend`] on [`App`]
pub trait ScatterApp {
    /// Set the backend for collecting [`AssetPart`]s, from which [`ScatterAssetCreationRequest`]s will be created.
    ///
    /// Only one backend can be set at a time for each source Material `T`.
    /// Setting a backend will replace any existing backend for the same Material type `T`. By default, no backend is set.
    ///
    /// The backend is supposed to return [`AssetPart`], which describes the relations of a discovered part.
    fn set_scatter_asset_backend<M>(
        &mut self,
        system: impl IntoSystem<In<()>, Result<Vec<AssetPart>>, M> + 'static,
    ) -> &mut App;

    fn configure_scatter_set<M: ScheduleLabel + Default>(&mut self) -> &mut App;

    fn configure_scatter_sets<'a>(&mut self) -> &mut App;
}

impl ScatterApp for App {
    fn set_scatter_asset_backend<M>(
        &mut self,
        system: impl IntoSystem<In<()>, Result<Vec<AssetPart>>, M> + 'static,
    ) -> &mut App {
        let id = self.register_system(system);
        self.world_mut().insert_resource(ScatterAssetBackend(id));
        self
    }

    fn configure_scatter_set<M: ScheduleLabel + Default>(&mut self) -> &mut App {
        self.configure_sets(
            M::default(),
            (
                ScatterSet::Loading.run_if(in_state(ScatterState::Loading)),
                ScatterSet::Setup.run_if(in_state(ScatterState::Setup)),
                ScatterSet::Collecting.run_if(in_state(ScatterState::Collecting)),
                (ScatterSet::Ready, ScatterSet::Clean)
                    .chain()
                    .run_if(in_state(ScatterState::Ready)),
            ),
        )
    }

    fn configure_scatter_sets<'a>(&mut self) -> &mut App {
        self.configure_scatter_set::<PreUpdate>()
            .configure_scatter_set::<Update>()
            .configure_scatter_set::<PostUpdate>()
    }
}
