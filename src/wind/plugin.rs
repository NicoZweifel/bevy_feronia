use crate::prelude::*;
use super::systems::*;
use bevy::asset::Asset;
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::load_shader_library;
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

pub struct WindMaterialPlugin<B, T>
where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    pub _marker: PhantomData<(B, T)>,
}

impl<B, T> Default for WindMaterialPlugin<B, T>
where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<B, T> Plugin for WindMaterialPlugin<B, T>
where
    B: Material,
    T: WindAffectable<B, T, WindAffectedTypes<T>, WindAffectedType<T>> + Asset + Clone,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<WindAffectedTypes<T>>().add_systems(
            Update,
            (
                collect_types::<B, T>,
                insert_material::<B, T>,
                update_materials::<B, T>.run_if(resource_changed::<Wind>),
            ),
        );
    }
}
