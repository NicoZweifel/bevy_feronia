use crate::prelude::*;
use bevy::{
    ecs::system::{SystemParamItem, lifetimeless::*},
    pbr::{
        RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewBindingArrayBindGroup,
    },
    render::{
        mesh::{RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
    },
};

pub struct SetInstancedWindAffectedMaterialBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P>
    for SetInstancedWindAffectedMaterialBindGroup<I>
{
    type Param = SRes<RenderAssets<PreparedInstancedWindAffectedMaterial>>;

    type ViewQuery = ();

    type ItemQuery = Read<InstancedWindAffectedMeshMaterial>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        items: Option<&'w InstancedWindAffectedMeshMaterial>,
        prepared_materials: SystemParamItem<'w, '_, Self::Param>,

        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(material) = items else {
            return RenderCommandResult::Skip;
        };

        let Some(prepared_material) = prepared_materials.into_inner().get(&material.0) else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &prepared_material.bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub struct SetInstancedWindAffectedPatchBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetInstancedWindAffectedPatchBindGroup<I> {
    type Param = ();

    type ViewQuery = ();

    type ItemQuery = Read<InstanceUniformBuffer>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        items: Option<&'w InstanceUniformBuffer>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance_uniform_buffer) = items else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &instance_uniform_buffer.bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub type DrawInstancedWindAffected = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    SetInstancedWindAffectedMaterialBindGroup<3>,
    SetInstancedWindAffectedPatchBindGroup<4>,
    DrawInstancedWindAffectedMesh,
);

pub struct DrawInstancedWindAffectedMesh;

impl<P: PhaseItem> RenderCommand<P> for DrawInstancedWindAffectedMesh {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );

    type ViewQuery = ();

    type ItemQuery = (Read<InstanceBuffer>, Option<Read<GpuDrawIndexedIndirect>>);

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        items: Option<(&'w InstanceBuffer, Option<&'w GpuDrawIndexedIndirect>)>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some((instance_buffer, indirect_draw_opt)) = items else {
            return RenderCommandResult::Skip;
        };

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };

        let mesh_allocator = mesh_allocator.into_inner();
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count: _,
            } => {
                let Some(indirect_draw) = indirect_draw_opt else {
                    return RenderCommandResult::Skip;
                };

                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);

                pass.draw_indexed_indirect(&indirect_draw.buffer, indirect_draw.offset);
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
