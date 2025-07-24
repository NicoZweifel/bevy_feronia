use bevy::prelude::*;
use bevy::render::render_resource::{BufferInitDescriptor, BufferUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use crate::prelude::*;

pub(crate) fn prepare_instance_buffer(
    mut cmd: Commands,
    query: Query<(Entity, &InstanceMaterialData, Option<&InstanceBuffer>)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (entity, instance_data, instance_buffer) in &query {
        let Some(instance_buffer) = instance_buffer else {
           create_buffer(&mut cmd, entity, instance_data, &render_device);
            continue;
        };

        if instance_data.len() != instance_buffer.length {
            create_buffer(&mut cmd, entity, instance_data, &render_device);
            continue;
        }

        render_queue.write_buffer(
            &instance_buffer.buffer,
            0,
            bytemuck::cast_slice(instance_data.as_slice()),
        );
    }
}

fn create_buffer(
    cmd: &mut Commands,
    entity: Entity,
    instance_data: &InstanceMaterialData,
    render_device: &Res<RenderDevice>,
) {
    let contents = bytemuck::cast_slice(instance_data.as_slice());

    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("instance data buffer"),
        contents,
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });

    cmd.entity(entity).insert(InstanceBuffer {
        buffer,
        length: instance_data.len(),
    });
}
