use super::systems::*;
use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct HeightMapPlugin;

impl Plugin for HeightMapPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "height_map.wgsl");

        app.init_resource::<HeightMapConfig>()
            .add_plugins(MaterialPlugin::<HeightMapMaterial>::default())
            .add_systems(Startup, (setup_materials, setup_height_map_pipeline))
            .add_systems(Update, create_height_map_ghost);
    }
}
