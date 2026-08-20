use super::systems::*;
use crate::prelude::*;

use bevy_app::{App, Plugin, PreUpdate, Startup, Update};
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

        app.register_type::<GlobalWind>()
            .register_type::<Wind>()
            .init_resource::<GlobalWind>()
            .configure_sets(Startup, WindTextureSetup)
            .add_systems(Startup, setup_wind_texture.in_set(WindTextureSetup))
            .add_systems(
                PreUpdate,
                (
                    sync_wind_preset.run_if(resource_changed::<GlobalWind>),
                    cycle_wind_history,
                )
                    .chain(),
            );
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
            (update_materials::<T>.run_if(
                resource_changed::<GlobalWind>
                    .or_else(material_options_changed)
                    .and_then(resource_exists::<Assets<ScatterAsset<T>>>)
                    .and_then(resource_exists::<ScatterAssetManager<T>>),
            ),),
        );
    }
}

// TODO granular updates && updating base color / properties (unlit)
fn material_options_changed(
    q_layer_changed: Query<(), (Or<(MaterialChangedFilter, WindChangedFilter)>,)>,
    q_root_changed: Query<
        (),
        (
            With<ScatterRoot>,
            Or<(MaterialChangedFilter, WindChangedFilter)>,
        ),
    >,
) -> bool {
    !q_layer_changed.is_empty() || !q_root_changed.is_empty()
}
