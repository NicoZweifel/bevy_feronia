use crate::prelude::*;

use crate::core::events::SpawnScatterAssets;
use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_eidolon::prelude::*;
use bevy_shader::load_shader_library;

pub struct InstancedWindAffectedPlugin;

impl Plugin for InstancedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "material.wgsl");
        load_shader_library!(app, "bindings.wgsl");
        embedded_asset!(app, "instanced.wgsl");
        embedded_asset!(app, "prepass.wgsl");

        app.add_plugins((
            InstancedMaterialCorePlugin,
            InstancedMaterialPlugin::<InstancedWindAffectedMaterial>::default(),
        ))
        .add_message::<SpawnScatterAssets<InstancedWindAffectedMaterial>>()
        .add_plugins(ScatterMaterialPlugin::<InstancedWindAffectedMaterial>::default());
    }
}

pub struct InstancedWindAffectedScatterPlugin;

impl Plugin for InstancedWindAffectedScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedWindAffectedPlugin,
            ScatterAssetRequestPlugin::<InstancedWindAffectedMaterial>::new(),
            ScatterAssetPlugin::<InstancedWindAffectedMaterial>::new(),
        ));
    }
}
