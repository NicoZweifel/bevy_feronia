use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;
use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_pbr::MaterialPlugin;

pub struct ExtendedWindAffectedPlugin;

impl Plugin for ExtendedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
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
