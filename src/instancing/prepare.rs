use crate::instancing::pipeline::InstancedWindAffectedPipeline;
use crate::prelude::*;
use bevy::pbr::RenderMeshInstances;
use bevy::prelude::*;
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BindGroupEntry, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::sync_world::MainEntity;
use bytemuck::bytes_of;

pub(crate) fn prepare_instance_buffer(
    mut cmd: Commands,
    query: Query<(Entity, &InstanceMaterialData, Option<&InstanceBuffer>)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (entity, instance_data, instance_buffer) in &query {
        let instance_vec = &instance_data.instances;

        let Some(instance_buffer) = instance_buffer else {
            create_buffer(&mut cmd, entity, instance_vec, &render_device);
            continue;
        };

        if instance_vec.len() != instance_buffer.length {
            create_buffer(&mut cmd, entity, instance_vec, &render_device);
            continue;
        }

        render_queue.write_buffer(
            &instance_buffer.buffer,
            0,
            bytemuck::cast_slice(instance_vec.as_slice()),
        );
    }
}

fn create_buffer(
    cmd: &mut Commands,
    entity: Entity,
    instance_vec: &Vec<InstanceData>,
    render_device: &Res<RenderDevice>,
) {
    let contents = bytemuck::cast_slice(instance_vec.as_slice());

    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("instance data buffer"),
        contents,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });

    cmd.entity(entity).insert(InstanceBuffer {
        buffer,
        length: instance_vec.len(),
    });
}

pub fn prepare_instance_uniform_buffer(
    mut cmd: Commands,
    query: Query<(
        Entity,
        &InstanceMaterialData,
        Option<&InstanceUniformBuffer>,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<InstancedWindAffectedPipeline>,
) {
    let bind_group_layout = pipeline.instance_uniform_layout.clone();

    for (entity, instance_data, uniform_buffer_opt) in &query {
        let uniforms = InstanceUniforms {
            color: instance_data.color,
            visibility_range: instance_data.visibility_range,
            static_bend_strength: instance_data.static_bend_strength,
            ..default()
        };
        let contents = bytes_of(&uniforms);

        if let Some(uniform_buffer) = uniform_buffer_opt {
            render_queue.write_buffer(&uniform_buffer.buffer, 0, contents);
        } else {
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("instance uniform buffer"),
                contents,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let bind_group = render_device.create_bind_group(
                "instance_uniform_bind_group",
                &bind_group_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            );

            cmd.entity(entity)
                .insert(InstanceUniformBuffer { buffer, bind_group });
        }
    }
}

pub fn prepare_indirect_draw_buffer(
    mut cmd: Commands,
    query: Query<
        (
            Entity,
            &MainEntity,
            &InstanceBuffer,
            Option<&GpuDrawIndexedIndirect>,
        ),
        With<InstancedWindAffectedMeshMaterial>,
    >,
    render_mesh_instances: Res<RenderMeshInstances>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mesh_allocator: Res<MeshAllocator>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (entity, main_entity, instance_buffer, indirect_buffer_opt) in &query {
        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity) else {
            continue;
        };
        let mesh_asset_id = mesh_instance.mesh_asset_id;

        let Some(gpu_mesh) = meshes.get(mesh_asset_id) else {
            continue;
        };
        let Some(vertex_buffer_slice) = mesh_allocator.mesh_vertex_slice(&mesh_asset_id) else {
            continue;
        };

        if let RenderMeshBufferInfo::Indexed { count, .. } = gpu_mesh.buffer_info {
            let Some(index_buffer_slice) = mesh_allocator.mesh_index_slice(&mesh_asset_id) else {
                continue;
            };

            let command = DrawIndexedIndirectArgs {
                index_count: count,
                instance_count: instance_buffer.length as u32,
                first_index: index_buffer_slice.range.start,
                base_vertex: vertex_buffer_slice.range.start as i32,
                first_instance: 0,
            };

            let contents = command.as_bytes();

            if let Some(indirect_buffer) = indirect_buffer_opt {
                render_queue.write_buffer(&indirect_buffer.buffer, 0, contents);
            } else {
                let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("draw_indexed_indirect buffer"),
                    contents,
                    usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST,
                });

                cmd.entity(entity)
                    .insert(GpuDrawIndexedIndirect { buffer, offset: 0 });
            }
        }
    }
}
