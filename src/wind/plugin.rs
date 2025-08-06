use super::systems::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::load_shader_library;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        load_shader_library!(app, "wind.wgsl");
        load_shader_library!(app, "bindings.wgsl");
        load_shader_library!(app, "noise.wgsl");
        load_shader_library!(app, "displace.wgsl");

        app.init_resource::<Wind>()
            .register_type::<Wind>()
            .add_systems(Startup, setup_wind_texture);
    }
}

pub struct WindMaterialPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    pub _marker: PhantomData<(TIn, TOut)>,
}

impl<TIn, TOut> Default for WindMaterialPlugin<TIn, TOut>
where
    TIn: Material,
    TOut: WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<TIn, TOut> Plugin for WindMaterialPlugin<TIn, TOut>
where
    TIn: Material,
    TOut:
        WindAffectable<ScatterAssets<TOut>, ScatterAsset<TOut>, TIn, TOut> + Asset + Clone + Debug,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<ScatterAssets<TOut>>()
            .init_resource::<ScatterAssetsNameMap<TOut>>()
            .add_systems(
                Update,
                (
                    collect_types::<TIn, TOut>,
                    update_name_map::<TOut>.run_if(resource_changed::<ScatterAssets<TOut>>),
                    insert_material::<TIn, TOut>,
                    update_materials::<TIn, TOut>.run_if(resource_changed::<Wind>),
                ),
            );
    }
}
