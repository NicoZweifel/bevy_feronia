use super::systems::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::shader::load_shader_library;
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

pub struct ScatterMaterialPlugin<TOut, TIn = StandardMaterial>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    pub _marker: PhantomData<(TOut, TIn)>,
}

impl<TOut, TIn> Default for ScatterMaterialPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone,
    TIn: Material,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<TOut, TIn> Plugin for ScatterMaterialPlugin<TOut, TIn>
where
    TOut: ScatterMaterial<TIn> + Asset + Clone + Debug,
    TIn: Material,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                replace_materials::<TOut, TIn>,
                update_materials::<TOut, TIn>.run_if(resource_changed::<Wind>),
            ),
        );
    }
}
