use crate::WindMaterialPlugin;
use crate::extension::material::ExtendedWindAffectedMaterial;
use crate::prelude::*;
use bevy::app::{App, Plugin};
use bevy::asset::embedded_asset;
use bevy::pbr::{MaterialPlugin, StandardMaterial};

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
