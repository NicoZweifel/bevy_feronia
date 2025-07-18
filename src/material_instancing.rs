use bevy::{
    core_pipeline::core_3d::{Opaque3d, Transparent3d},
    ecs::{
        query::QueryItem,
        system::{SystemParamItem, lifetimeless::*},
    },
    pbr::{
        MaterialPipeline, MaterialPipelineKey, MeshPipeline, MeshPipelineKey, RenderMeshInstances,
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        mesh::{
            MeshVertexBufferLayoutRef, RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator,
        },
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::ExtractedView,
    },
};
use bytemuck::{Pod, Zeroable};

use crate::{WindPlugin, prelude::*};

const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1382378492386749823);

pub struct InstancedMaterialPlugin;

impl Plugin for InstancedMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<InstancedWindAffectedMaterial>::default());
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            &SHADER_HANDLE,
            Shader::from_wgsl(
                include_str!("../assets/shaders/instancing.wgsl"),
                "shaders/instancing.wgsl",
            ),
        );

        app.add_plugins(MaterialPlugin::<InstancedWindAffectedMaterial>::default())
            .add_plugins(WindPlugin::<StandardMaterial, InstancedWindAffectedMaterial>::default());
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}

#[derive(Component, Clone, Deref)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, WindUniform, binding_array(10))]
#[bindless(limit(4))]
pub struct InstancedWindAffectedMaterial {
    pub wind: Wind,

    #[texture(1)]
    #[sampler(2)]
    pub noise_texture: Handle<Image>,
}

impl Material for InstancedWindAffectedMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_HANDLE.into()
    }

    fn specialize(
        pipeline: &MaterialPipeline<Self>,
        _descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut descriptor = pipeline.specialize(key, layout)?;
        let vertex_layout = descriptor.vertex.buffers.get_mut(0).unwrap();
        vertex_layout.step_mode = VertexStepMode::Instance;
        Ok(())
    }
}

impl WindAffectable<StandardMaterial, InstancedWindAffectedMaterial>
    for InstancedWindAffectedMaterial
{
    fn create_material(
        _: StandardMaterial,
        wind: Wind,
        noise_texture: Handle<Image>,
    ) -> InstancedWindAffectedMaterial {
        InstancedWindAffectedMaterial {
            wind,
            noise_texture,
        }
    }

    fn update_material(mut materials: ResMut<Assets<InstancedWindAffectedMaterial>>, wind: Wind) {
        for (_, material) in materials.iter_mut() {
            material.wind = wind.clone();
        }
    }
}

impl<'a> From<&'a InstancedWindAffectedMaterial> for WindUniform {
    fn from(material: &'a InstancedWindAffectedMaterial) -> Self {
        WindUniform::from(&material.wind)
    }
}
