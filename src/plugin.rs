use crate::prelude::*;
use crate::systems;
use bevy::asset::Asset;
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::load_shader_library;
use std::marker::PhantomData;
use bevy::render::RenderSystems::Prepare;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "wind.wgsl");
        load_shader_library!(app, "displace.wgsl");

        app.init_resource::<Wind>()
            .register_type::<Wind>()
            .add_systems(Startup, systems::setup_wind_texture);
    }
}

pub struct WindMaterialPlugin<M: Material, W: WindAffectable<M, W> + Asset> {
    pub _marker: PhantomData<(M, W)>,
}

impl<M: Material, W: WindAffectable<M, W> + Asset> Default for WindMaterialPlugin<M, W> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<M: Material, W: WindAffectable<M, W> + Asset> Plugin for WindMaterialPlugin<M, W> {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindAffectedTypes<W>>()
            .add_systems(
                Update,
                (systems::update_materials::<M, W>.run_if(resource_changed::<Wind>),),
            )
            .add_systems(PostUpdate, (systems::setup_wind_affected::<M, W>,).before(Prepare));
    }
}
