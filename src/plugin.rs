use bevy::pbr::Material;
use bevy::asset::Asset;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{resource_changed, IntoScheduleConfigs};
use std::marker::PhantomData;
use bevy::render::load_shader_library;
use crate::prelude::*;
use crate::systems;

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
        app.init_resource::<WindAffectedTypes<W>>().add_systems(
            Update,
            (
                systems::setup_wind_affected::<M, W>,
                systems::update_materials::<M, W>.run_if(resource_changed::<Wind>),
            ),
        );
    }
}
