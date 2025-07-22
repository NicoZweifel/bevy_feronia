use crate::prelude::*;
use crate::systems::*;
use bevy::asset::Asset;
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::RenderSystems::Prepare;
use bevy::render::load_shader_library;
use std::marker::PhantomData;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "types.wgsl");
        load_shader_library!(app, "wind.wgsl");
        load_shader_library!(app, "displace.wgsl");

        app.init_resource::<Wind>()
            .register_type::<Wind>()
            .add_systems(Startup, setup_wind_texture);
    }
}

pub struct WindMaterialPlugin<M, W>
where
    M: Material,
    W: WindAffectable<M, W> + Asset,
{
    pub _marker: PhantomData<(M, W)>,
}

impl<M, W> Default for WindMaterialPlugin<M, W>
where
    M: Material,
    W: WindAffectable<M, W> + Asset,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<M, W> Plugin for WindMaterialPlugin<M, W>
where
    M: Material,
    W: WindAffectable<M, W> + Asset + Clone,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<WindAffectedTypes<W>>().add_systems(
            Update,
            (
                collect_types::<M, W>,
                insert_material::<M, W>,
                update_materials::<M, W>.run_if(resource_changed::<Wind>),
            ),
        );
    }
}
