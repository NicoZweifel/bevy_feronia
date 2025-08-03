use super::{components::InstanceData, material::InstancedWindAffectedMaterial};
use crate::prelude::WindAffectedKey;
use bevy::{
    asset::{AssetPath, embedded_path},
    pbr::{MeshPipeline, MeshPipelineKey},
    prelude::*,
    render::{mesh::MeshVertexBufferLayoutRef, render_resource::*, renderer::RenderDevice},
};
use std::mem::size_of;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CustomPipelineKey {
    pub(crate) mesh_key: MeshPipelineKey,
    pub(crate) wind_key: WindAffectedKey,
}

#[derive(Resource)]
pub(crate) struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    material_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>().clone();
        let render_device = world.resource::<RenderDevice>();
        let material_layout = InstancedWindAffectedMaterial::bind_group_layout(render_device);
        let asset_server = world.resource::<AssetServer>();

        CustomPipeline {
            shader: asset_server.load(
                AssetPath::from_path_buf(embedded_path!("instancing.wgsl")).with_source("embedded"),
            ),
            mesh_pipeline,
            material_layout,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = CustomPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;
        descriptor.layout.push(self.material_layout.clone());

        let shader_defs = &mut descriptor.vertex.shader_defs;
        if key.wind_key.contains(WindAffectedKey::ENABLE_BILLBOARDING) {
            shader_defs.push("WIND_BILLBOARDING".into());
        }
        if key
            .wind_key
            .contains(WindAffectedKey::ENABLE_EDGE_CORRECTION)
        {
            shader_defs.push("WIND_EDGE_CORRECTION".into());
        }
        if key.wind_key.contains(WindAffectedKey::HIGH_QUALITY) {
            shader_defs.push("WIND_HIGH_QUALITY".into());
        }
        if key.wind_key.contains(WindAffectedKey::FAST_NORMALS) {
            shader_defs.push("FAST_NORMALS".into());
        }

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // Position + Scale
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 8,
                },
                // Color
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 9,
                },
                // Index
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: VertexFormat::Float32x4.size() * 2,
                    shader_location: 10,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        Ok(descriptor)
    }
}
