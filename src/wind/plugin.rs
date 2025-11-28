use super::systems::*;
use crate::prelude::*;

use bevy_app::{App, Plugin, Startup, Update};
use bevy_asset::Assets;
use bevy_ecs::prelude::*;
use bevy_pbr::StandardMaterial;
use bevy_shader::load_shader_library;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct WindPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindTextureSetup;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        load_shader_library!(app, "wind.wgsl");
        load_shader_library!(app, "bindings.wgsl");
        load_shader_library!(app, "noise.wgsl");
        load_shader_library!(app, "displace.wgsl");
        load_shader_library!(app, "forward_sss_io.wgsl");

        app.init_resource::<Wind>()
            .register_type::<Wind>()
            .configure_sets(Startup, WindTextureSetup)
            .add_systems(Startup, setup_wind_texture.in_set(WindTextureSetup));
    }
}

pub struct ScatterMaterialPlugin<T = StandardMaterial>
where
    T: ScatterMaterial,
{
    pub _marker: PhantomData<T>,
}

impl<T> Default for ScatterMaterialPlugin<T>
where
    T: ScatterMaterial,
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> ScatterMaterialPlugin<T>
where
    T: ScatterMaterial,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Plugin for ScatterMaterialPlugin<T>
where
    T: ScatterMaterial,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_materials::<T>
                .run_if(resource_changed::<Wind>.and(resource_exists::<Assets<ScatterAsset<T>>>)),),
        );
    }
}
