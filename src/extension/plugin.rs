use crate::prelude::*;
use bevy::prelude::*;
use bevy::asset::embedded_asset;

pub struct ExtendedWindAffectedPlugin;

impl Plugin for ExtendedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "main.wgsl");
        embedded_asset!(app, "prepass.wgsl");

        app.add_plugins(MaterialPlugin::<ExtendedWindAffectedMaterial>::default())
            .add_plugins(WindMaterialPlugin::<
                StandardMaterial,
                ExtendedWindAffectedMaterial,
            >::default());
    }
}
