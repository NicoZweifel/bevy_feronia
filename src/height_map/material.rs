use crate::prelude::HeightMapConfig;
use bevy_asset::*;
use bevy_ecs::prelude::*;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::*;
use bevy_reflect::TypePath;
use bevy_render::render_resource::*;
use bevy_shader::ShaderRef;

#[repr(C)]
#[derive(Clone, ShaderType, Debug)]
pub struct HeightSettings {
    pub min_height: f32,
    pub max_height: f32,
}

impl From<&HeightMapConfig> for HeightSettings {
    fn from(value: &HeightMapConfig) -> Self {
        Self {
            max_height: value.world_height_range.end,
            min_height: value.world_height_range.start,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HeightMapMaterial {
    #[uniform(0)]
    pub height_settings: HeightSettings,
}

impl From<&HeightMapConfig> for HeightMapMaterial {
    fn from(value: &HeightMapConfig) -> Self {
        Self {
            height_settings: HeightSettings::from(value),
        }
    }
}

impl Material for HeightMapMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("height_map.wgsl")).with_source("embedded"),
        )
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("height_map.wgsl")).with_source("embedded"),
        )
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}
