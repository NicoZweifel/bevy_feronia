use super::systems::*;
use crate::height_map::state::HeightMapState;
use crate::prelude::*;

use bevy_app::*;
use bevy_asset::embedded_asset;
use bevy_ecs::prelude::*;
use bevy_pbr::MaterialPlugin;
use bevy_state::prelude::*;
use bevy_transform::TransformSystems;

pub struct HeightMapPlugin;

impl Plugin for HeightMapPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "height_map.wgsl");

        app.init_state::<HeightMapState>()
            .add_plugins(MaterialPlugin::<HeightMapMaterial>::default())
            .add_systems(
                PostUpdate,
                setup_config
                    .chain()
                    .after(TransformSystems::Propagate)
                    .run_if(
                        not(resource_exists::<HeightMapConfig>)
                            .and(in_state(HeightMapState::Setup)),
                    ),
            )
            .add_systems(
                Update,
                (
                    skip_setup.run_if(
                        not(resource_exists::<HeightMapConfig>)
                            .and(in_state(HeightMapState::Setup)),
                    ),
                    ((setup_materials, setup_height_map_pipeline), finish_setup)
                        .chain()
                        .run_if(
                            resource_exists::<HeightMapConfig>.and(in_state(HeightMapState::Setup)),
                        ),
                    create_height_map_ghost.run_if(
                        resource_exists::<HeightMapConfig>.and(in_state(HeightMapState::Ghost)),
                    ),
                    bake_height_map.run_if(in_state(HeightMapState::Baking)),
                ),
            )
            .add_systems(OnEnter(HeightMapState::Ready), teardown_height_map_pipeline);
    }
}
