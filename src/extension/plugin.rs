use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct ExtendedWindAffectedPlugin;

impl Plugin for ExtendedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "main.wgsl");
        embedded_asset!(app, "prepass.wgsl");

        app.add_plugins(MaterialPlugin::<ExtendedWindAffectedMaterial>::default())
            .add_message::<SpawnProtoTypes<ExtendedWindAffectedMaterial>>()
            .add_plugins(ScatterMaterialPlugin::<ExtendedWindAffectedMaterial>::default())
            .add_systems(Update, ExtendedWindAffectedMaterial::spawn);
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
