use crate::prelude::*;
use bevy::asset::{AssetPath, embedded_path};
use bevy::camera::primitives::Aabb;
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

    pub aabb: Aabb,

    pub options: MaterialOptions,

    #[texture(51)]
    #[sampler(52)]
    pub noise_texture: Handle<Image>,
}

impl<'a> From<&'a WindAffectedExtension> for WindUniform {
    fn from(material_extension: &'a WindAffectedExtension) -> Self {
        WindUniform::from(&material_extension.wind)
            .with_lod_threshold(material_extension.options.lod_threshold)
            .with_round_exponent(material_extension.options.round_exponent)
            .with_edge_correction_factor(material_extension.options.edge_correction_factor)
            .with_aabb(&material_extension.aabb)
            .with_debug_color(material_extension.options.debug_color.to_linear().to_vec4())
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

        if key.bind_group_data.contains(WindAffectedKey::WIND_LOW_QUALITY) {
            shader_defs.push("WIND_LOW_QUALITY".into());
        }

        if key.bind_group_data.contains(WindAffectedKey::FAST_NORMALS) {
            shader_defs.push("FAST_NORMALS".into());
        }

        if key.bind_group_data.contains(WindAffectedKey::DEBUG) {
            shader_defs.push("MATERIAL_DEBUG".into());
            descriptor
                .fragment
                .as_mut()
                .unwrap()
                .shader_defs
                .push("MATERIAL_DEBUG".into());
        }

        Ok(())
    }
}

impl From<&WindAffectedExtension> for WindAffectedKey {
    fn from(material: &WindAffectedExtension) -> Self {
        let mut key = WindAffectedKey::empty();
        key.set(WindAffectedKey::DEBUG, material.options.debug);
        key.set(
            WindAffectedKey::ENABLE_BILLBOARDING,
            material.options.enable_billboarding,
        );
        key.set(
            WindAffectedKey::ENABLE_EDGE_CORRECTION,
            material.options.edge_correction_factor > 0.,
        );
        key.set(WindAffectedKey::WIND_LOW_QUALITY, material.wind.low_quality);
        key.set(WindAffectedKey::FAST_NORMALS, material.options.fast_normals);

        key
    }
}
