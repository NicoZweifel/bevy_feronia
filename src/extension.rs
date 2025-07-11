use bevy::{
    pbr::MaterialExtension,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::prelude::*;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WindAffectedExtension {
    #[uniform(100)]
    pub wind: Wind,

    #[texture(101)]
    #[sampler(102)]
    pub noise_texture: Handle<Image>,
}

const SHADER_MAIN_ASSET_PATH: &str = "shaders/wind_main.wgsl";
const SHADER_PREPASS_ASSET_PATH: &str = "shaders/wind_prepass.wgsl";

impl MaterialExtension for WindAffectedExtension {
    fn vertex_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        SHADER_PREPASS_ASSET_PATH.into()
    }
}
