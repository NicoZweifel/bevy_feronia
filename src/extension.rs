use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{WindPlugin, prelude::*};

pub struct ExtendedMaterialPlugin;

impl Plugin for ExtendedMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WindAffectedExtendedMaterial>::default())
            .add_plugins(WindPlugin::<StandardMaterial, WindAffectedExtendedMaterial>::default());
    }
}

pub type WindAffectedExtendedMaterial = ExtendedMaterial<StandardMaterial, WindAffectedExtension>;

impl WindAffectable<StandardMaterial, WindAffectedExtendedMaterial>
    for WindAffectedExtendedMaterial
{
    fn create_material(
        base: StandardMaterial,
        wind: Wind,
        noise_texture: Handle<Image>,
    ) -> WindAffectedExtendedMaterial {
        ExtendedMaterial {
            base,
            extension: WindAffectedExtension {
                noise_texture,
                wind,
            },
        }
    }

    fn update_material(mut materials: ResMut<Assets<WindAffectedExtendedMaterial>>, wind: Wind) {
        for (_, material) in materials.iter_mut() {
            let ext = &mut material.extension;
            ext.wind = wind.clone();
        }
    }
}

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
#[data(50, WindUniform, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
pub struct WindAffectedExtension {
    pub wind: Wind,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl<'a> From<&'a WindAffectedExtension> for WindUniform {
    fn from(material_extension: &'a WindAffectedExtension) -> Self {
        WindUniform::from(&material_extension.wind)
    }
}

const SHADER_MAIN_ASSET_PATH: &str = "shaders/wind_main.wgsl";
const SHADER_PREPASS_ASSET_PATH: &str = "shaders/wind_prepass.wgsl";

impl MaterialExtension for WindAffectedExtension {
    fn fragment_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn vertex_shader() -> ShaderRef {
        SHADER_MAIN_ASSET_PATH.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        SHADER_PREPASS_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialExtensionPipeline,
        _descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialExtensionKey<Self>,
    ) -> std::result::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {



        Ok(())
    }
}
