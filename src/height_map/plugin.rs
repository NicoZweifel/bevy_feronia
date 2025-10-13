use super::systems::*;
use crate::height_map::state::HeightMapState;
use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct HeightMapPlugin;

impl Plugin for HeightMapPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "height_map.wgsl");

        app.init_state::<HeightMapState>()
            .add_plugins(MaterialPlugin::<HeightMapMaterial>::default())
            .add_systems(
                Update,
                (
                    (setup_config, skip_setup).run_if(
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
            .add_systems(OnEnter(HeightMapState::Ready), cleanup_height_map_pipeline);
    }
}
