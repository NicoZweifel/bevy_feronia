use crate::prelude::*;
use bevy::asset::{AssetPath, embedded_path};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
#[bind_group_data(WindAffectedKey)]
#[data(50, WindUniform, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
pub struct WindAffectedExtension {
    pub wind: Wind,
    // Whether the Extension is controlled externally and isn't automatically updated by the Wind resource.
    pub controlled: bool,

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
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("main.wgsl")).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("main.wgsl")).with_source("embedded"),
        )
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("prepass.wgsl")).with_source("embedded"),
        )
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let shader_defs = &mut descriptor.vertex.shader_defs;

        if key
            .bind_group_data
            .contains(WindAffectedKey::ENABLE_BILLBOARDING)
        {
            shader_defs.push("WIND_BILLBOARDING".into());
        }

        if key
            .bind_group_data
            .contains(WindAffectedKey::ENABLE_EDGE_CORRECTION)
        {
            shader_defs.push("WIND_EDGE_CORRECTION".into());
        }

        if key.bind_group_data.contains(WindAffectedKey::HIGH_QUALITY) {
            shader_defs.push("WIND_HIGH_QUALITY".into());
        }

        if key.bind_group_data.contains(WindAffectedKey::FAST_NORMALS) {
            shader_defs.push("FAST_NORMALS".into());
        }

        Ok(())
    }
}

impl From<&WindAffectedExtension> for WindAffectedKey {
    fn from(material: &WindAffectedExtension) -> Self {
        let mut key = WindAffectedKey::empty();

        key.set(
            WindAffectedKey::ENABLE_BILLBOARDING,
            material.wind.enable_billboarding,
        );
        key.set(
            WindAffectedKey::ENABLE_EDGE_CORRECTION,
            material.wind.enable_edge_correction,
        );
        key.set(WindAffectedKey::HIGH_QUALITY, material.wind.high_quality);
        key.set(WindAffectedKey::FAST_NORMALS, material.wind.fast_normals);

        key
    }
}
