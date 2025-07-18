use crate::{WindMaterialPlugin, prelude::*};
use bevy::asset::{load_internal_asset, weak_handle};
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct ExtendedMaterialPlugin;

const WIND_MAIN_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("d92c3b99-95cb-4b6b-9aef-998edc557668");
const WIND_PREPASS_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("fcb04db3-2018-4100-b8fa-e4bfb623de71");

impl Plugin for ExtendedMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            WIND_MAIN_SHADER_HANDLE,
            "main.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            WIND_PREPASS_SHADER_HANDLE,
            "prepass.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<WindAffectedExtendedMaterial>::default())
            .add_plugins(WindMaterialPlugin::<StandardMaterial, WindAffectedExtendedMaterial>::default());
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


impl MaterialExtension for WindAffectedExtension {
    fn fragment_shader() -> ShaderRef {
        WIND_MAIN_SHADER_HANDLE.into()
    }

    fn vertex_shader() -> ShaderRef {
        WIND_MAIN_SHADER_HANDLE.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        WIND_PREPASS_SHADER_HANDLE.into()
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
