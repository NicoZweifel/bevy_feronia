use bevy::app::{App, Plugin};
use bevy::asset::embedded_asset;
use bevy::pbr::{MaterialPlugin, StandardMaterial};
use crate::extension::material::WindAffectedExtendedMaterial;
use crate::WindMaterialPlugin;

pub struct ExtendedWindAffectedPlugin;

impl Plugin for ExtendedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "main.wgsl");
        embedded_asset!(app, "prepass.wgsl");

        app.add_plugins(MaterialPlugin::<WindAffectedExtendedMaterial>::default())
            .add_plugins(WindMaterialPlugin::<
                StandardMaterial,
                WindAffectedExtendedMaterial,
            >::default());
    }
}