use crate::extension::spawn::spawn_extended_wind_affected;
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
            .add_plugins(WindMaterialPlugin::<
                StandardMaterial,
                ExtendedWindAffectedMaterial,
            >::default())
            .add_systems(Update, spawn_extended_wind_affected);
    }
}

pub struct ExtendedWindAffectedScatterPlugin;

impl Plugin for ExtendedWindAffectedScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtendedWindAffectedPlugin,
            ScatterAssetPlugin::<StandardMaterial, ExtendedWindAffectedMaterial>::new(),
        ));
    }
}
