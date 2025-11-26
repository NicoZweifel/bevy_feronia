use crate::instancing::pipeline::{InstancedComputePipeline, InstancedWindAffectedPipeline};
use crate::instancing::resources::{CameraCullData, GlobalCullBuffer, GrassBufferCache};
use crate::prelude::*;
use bevy_camera::Camera;
use bevy_ecs::prelude::*;
use bevy_math::Vec4;
use bevy_pbr::RenderMeshInstances;
use bevy_render::mesh::allocator::MeshAllocator;
use bevy_render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_resource::{
    BindGroupEntry, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs,
};
use bevy_render::renderer::{RenderDevice, RenderQueue};
use bevy_render::sync_world::MainEntity;
use bevy_render::view::ExtractedView;
use bytemuck::bytes_of;

#[cfg(feature = "tracing")]
use tracing::warn;

pub(super) fn prepare_instance_buffer(
    mut cmd: Commands,
    query: Query<(Entity, &InstanceMaterialData, Option<&InstanceBuffer>), Without<GpuCull>>,
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

pub(super) fn prepare_instance_uniform_buffer(
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
        let uniforms: InstanceUniforms = instance_data.into();
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

pub(super) fn prepare_indirect_draw_buffer(
    mut cmd: Commands,
    query: Query<
        (
            Entity,
            &MainEntity,
            &InstanceBuffer,
            Option<&GpuDrawIndexedIndirect>,
        ),
        (With<InstancedWindAffectedMeshMaterial>, Without<GpuCull>),
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

pub(super) fn prepare_instanced_compute_source(
    mut commands: Commands,
    query: Query<
        (Entity, &MainEntity, &InstanceMaterialData),
        (Without<InstancedComputeSourceBuffer>, With<GpuCull>),
    >,
    render_device: Res<RenderDevice>,
    mut cache: ResMut<GrassBufferCache>,
) {
    for (entity, main_entity, instance_data) in &query {
        let count = instance_data.instances.len();
        if count == 0 {
            continue;
        }

        let buffer = if let Some(buffer) = cache.buffers.get(&**main_entity) {
            buffer.clone()
        } else {
            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("instanced_compute_source_buffer"),
                contents: bytemuck::cast_slice(&instance_data.instances),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

            cache.buffers.insert(**main_entity, buffer.clone());
            buffer
        };

        commands
            .entity(entity)
            .insert(InstancedComputeSourceBuffer {
                buffer,
                count: count as u32,
            });
    }
}

pub(super) fn prepare_instanced_compute_output(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData), (Without<InstanceBuffer>, With<GpuCull>)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let count = instance_data.instances.len();
        if count == 0 {
            continue;
        }

        let size = (count * size_of::<InstanceData>()) as u64;

        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("instanced_compute_output_buffer"),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: count,
        });
    }
}

pub(super) fn prepare_instanced_compute_indirect(
    mut commands: Commands,
    query: Query<(Entity, &MainEntity), (Without<GpuDrawIndexedIndirect>, With<GpuCull>)>,
    render_mesh_instances: Res<RenderMeshInstances>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mesh_allocator: Res<MeshAllocator>,
    render_device: Res<RenderDevice>,
) {
    for (entity, main_entity) in &query {
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
                instance_count: 0,
                first_index: index_buffer_slice.range.start,
                base_vertex: vertex_buffer_slice.range.start as i32,
                first_instance: 0,
            };

            let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("instanced_compute_indirect_buffer"),
                contents: command.as_bytes(),
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            });

            commands
                .entity(entity)
                .insert(GpuDrawIndexedIndirect { buffer, offset: 0 });
        }
    }
}

pub(super) fn prepare_instanced_compute_bind_group(
    mut cmd: Commands,
    query: Query<
        (
            Entity,
            &InstancedComputeSourceBuffer,
            &InstanceBuffer,
            &GpuDrawIndexedIndirect,
        ),
        (Without<InstancedComputeBindGroup>, With<GpuCull>),
    >,
    render_device: Res<RenderDevice>,
    pipeline: Res<InstancedComputePipeline>,
    global_cull_buffer: Option<Res<GlobalCullBuffer>>,
) {
    let Some(cull_buffer) = global_cull_buffer else {
        return;
    };

    for (entity, source, instance, indirect) in &query {
        let bind_group = render_device.create_bind_group(
            "instanced_compute_bind_group",
            &pipeline.layout,
            &[
                // Source
                BindGroupEntry {
                    binding: 0,
                    resource: source.buffer.as_entire_binding(),
                },
                // Instance Output
                BindGroupEntry {
                    binding: 1,
                    resource: instance.buffer.as_entire_binding(),
                },
                // Indirect Args
                BindGroupEntry {
                    binding: 2,
                    resource: indirect.buffer.as_entire_binding(),
                },
                // Camera
                BindGroupEntry {
                    binding: 3,
                    resource: cull_buffer.buffer.as_entire_binding(),
                },
            ],
        );

        cmd.entity(entity)
            .insert(InstancedComputeBindGroup(bind_group));
    }
}

pub(super) fn prepare_global_cull_buffer(
    mut commands: Commands,
    views: Query<(&ExtractedView, &Camera), With<Center>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    global_buffer: Option<ResMut<GlobalCullBuffer>>,
) {
    let Some((view, _camera)) = views.iter().find(|(_, cam)| cam.is_active) else {
        #[cfg(feature = "tracing")]
        warn!(
            "No active camera found culling. Did you add `Center` to the camera/player controller?"
        );
        return;
    };

    let view_proj = view.clip_from_world.unwrap_or_else(|| {
        let world_from_view = view.world_from_view.to_matrix();
        let view_from_world = world_from_view.inverse();
        view.clip_from_view * view_from_world
    });

    let camera_position = view.world_from_view.translation();

    let data = CameraCullData {
        view_proj,
        view_pos: Vec4::from((camera_position, 1.0)),
        settings: Vec4::new(50.0, 200.0, 0.0, 0.0),
    };

    let contents = bytes_of(&data);

    if let Some(global) = global_buffer {
        render_queue.write_buffer(&global.buffer, 0, contents);
    } else {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("global_cull_camera_buffer"),
            contents,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        commands.insert_resource(GlobalCullBuffer { buffer });
    }
}

pub(super) fn prepare_reset_indirect_buffer(
    query: Query<&GpuDrawIndexedIndirect, With<GpuCull>>,
    render_queue: Res<RenderQueue>,
) {
    for indirect in &query {
        // 'instance_count' is at offset 4 in DrawIndexedIndirectArgs.
        render_queue.write_buffer(&indirect.buffer, 4, &[0, 0, 0, 0]);
    }
}
