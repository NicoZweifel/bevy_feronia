use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;

use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_pbr::MaterialPlugin;
use bevy_shader::load_shader_library;

pub struct ExtendedWindAffectedPlugin;

impl Plugin for ExtendedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "material.wgsl");
        load_shader_library!(app, "bindings.wgsl");
        embedded_asset!(app, "fragment.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "prepass.wgsl");

        app.add_plugins(MaterialPlugin::<ExtendedWindAffectedMaterial>::default())
            .add_message::<SpawnScatterAssets<ExtendedWindAffectedMaterial>>()
            .add_plugins(ScatterMaterialPlugin::<ExtendedWindAffectedMaterial>::default());
    }
}

pub struct ExtendedWindAffectedScatterPlugin;

impl Plugin for ExtendedWindAffectedScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtendedWindAffectedPlugin,
            ScatterAssetsPlugin::<ExtendedWindAffectedMaterial>::new(),
            ScatterAssetPlugin::<ExtendedWindAffectedMaterial>::new(),
        ));
    }
}
