use super::{components::InstanceData, material::InstancedWindAffectedMaterial};
use crate::prelude::{InstanceUniforms, WindAffectedKey};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::{
    asset::{AssetPath, embedded_path},
    mesh::VertexBufferLayout,
    pbr::{MeshPipeline, MeshPipelineKey},
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};
use std::mem::size_of;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InstancedWindAffectedPipelineKey {
    pub mesh_key: MeshPipelineKey,
    pub wind_key: WindAffectedKey,
}

#[derive(Resource)]
pub struct InstancedWindAffectedPipeline {
    pub shader: Handle<Shader>,
    pub mesh_pipeline: MeshPipeline,
    pub material_layout: BindGroupLayout,
    pub instance_uniform_layout: BindGroupLayout,
}

impl FromWorld for InstancedWindAffectedPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>().clone();
        let render_device = world.resource::<RenderDevice>();
        let material_layout = InstancedWindAffectedMaterial::bind_group_layout(render_device);
        let asset_server = world.resource::<AssetServer>();

        let instance_uniform_layout = render_device.create_bind_group_layout(
            "instance_uniform_layout",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<InstanceUniforms>() as u64),
                },
                count: None,
            }],
        );

        InstancedWindAffectedPipeline {
            shader: asset_server.load(
                AssetPath::from_path_buf(embedded_path!("instancing.wgsl")).with_source("embedded"),
            ),
            mesh_pipeline,
            material_layout,
            instance_uniform_layout,
        }
    }
}

impl SpecializedMeshPipeline for InstancedWindAffectedPipeline {
    type Key = InstancedWindAffectedPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;

        descriptor.layout.push(self.material_layout.clone());
        descriptor.layout.push(self.instance_uniform_layout.clone());

        if let Some(ds) = descriptor.depth_stencil.as_mut() {
            ds.depth_write_enabled = true;
            ds.depth_compare = CompareFunction::GreaterEqual;
        }

        if !key.wind_key.contains(WindAffectedKey::BILLBOARDING) {
            descriptor.primitive.cull_mode = None;
        }

        if let Some(fragment) = descriptor.fragment.as_mut()
            && let Some(target) = fragment.targets.get_mut(0)
            && let Some(target) = target
        {
            target.blend = None;
        }

        let shader_defs = &mut descriptor.vertex.shader_defs;

        if !shader_defs.contains(&"MAY_DISCARD".into()) {
            shader_defs.push("MAY_DISCARD".into());
        }

        shader_defs.push("VISIBILITY_RANGE_DITHER".into());

        if key.wind_key.contains(WindAffectedKey::BILLBOARDING) {
            shader_defs.push("BILLBOARDING".into());
        }

        if key.wind_key.contains(WindAffectedKey::EDGE_CORRECTION) {
            shader_defs.push("EDGE_CORRECTION".into());
        }

        if key.wind_key.contains(WindAffectedKey::WIND_LOW_QUALITY) {
            shader_defs.push("WIND_LOW_QUALITY".into());
        }

        if key.wind_key.contains(WindAffectedKey::FAST_NORMALS) {
            shader_defs.push("FAST_NORMALS".into());
        }

        if key.wind_key.contains(WindAffectedKey::WIND_AFFECTED) {
            shader_defs.push("WIND_AFFECTED".into());
        }

        if key.wind_key.contains(WindAffectedKey::DEBUG) {
            shader_defs.push("MATERIAL_DEBUG".into());
        }

        if key.wind_key.contains(WindAffectedKey::STATIC_BEND) {
            shader_defs.push("STATIC_BEND".into());
        }

        if key.wind_key.contains(WindAffectedKey::ANALYTICAL_NORMALS) {
            shader_defs.push("ANALYTICAL_NORMALS".into());
        }

        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.push("VISIBILITY_RANGE_DITHER".into());

            if key.wind_key.contains(WindAffectedKey::CURVE_NORMALS) {
                fragment.shader_defs.push("CURVE_NORMALS".into());
            }

            if key.wind_key.contains(WindAffectedKey::POINT_LIGHTS) {
                fragment.shader_defs.push("POINT_LIGHTS".into());
            }
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
                // Index
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 9,
                },
            ],
        });

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        Ok(descriptor)
    }
}
