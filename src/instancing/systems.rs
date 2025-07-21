use super::{
    components::{InstanceBuffer, InstanceMaterialData, InstancePipelineKey},
    draw::DrawCustom,
    material::{InstancedWindAffectedMaterial, InstancedWindAffectedMeshMaterial},
    pipeline::{CustomPipeline, CustomPipelineKey},
};
use crate::prelude::*;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    pbr::{MeshPipelineKey, RenderMeshInstances},
    prelude::*,
    render::{
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, PhaseItemExtraIndex, ViewSortedRenderPhases},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
        view::ExtractedView,
    },
};

pub(crate) fn add_instance_key_component(
    mut commands: Commands,
    materials: Res<Assets<InstancedWindAffectedMaterial>>,
    query: Query<(Entity, &InstancedWindAffectedMeshMaterial), Without<InstancePipelineKey>>,
) {
    for (entity, material_handle) in &query {
        let Some(material) = materials.get(&material_handle.0) else {
            continue;
        };
        let mut key = WindAffectedKey::empty();
        key.set(
            WindAffectedKey::ENABLE_BILLBOARDING,
            material.wind.enable_billboarding,
        );
        key.set(
            WindAffectedKey::ENABLE_EDGE_CORRECTION,
            material.wind.enable_edge_correction,
        );
        key.set(WindAffectedKey::ENABLE_LOD, material.wind.enable_lod);

        commands
            .entity(entity)
            .insert(InstancePipelineKey(key.bits()));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<
        (Entity, &MainEntity, &InstancePipelineKey),
        (
            With<InstanceMaterialData>,
            With<InstancedWindAffectedMeshMaterial>,
        ),
    >,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };
        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (entity, main_entity, instance_key) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let key = CustomPipelineKey {
                mesh_key: view_key
                    | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology()),
                wind_key: WindAffectedKey::from_bits(instance_key.0).unwrap(),
            };
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder.distance_translation(&mesh_instance.translation),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

pub(crate) fn prepare_instance_buffers(
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
