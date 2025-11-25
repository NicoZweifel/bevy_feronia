use super::{
    components::{InstanceMaterialData, InstancePipelineKey},
    draw::DrawInstancedWindAffected,
    material::{InstancedWindAffectedMaterial, InstancedWindAffectedMeshMaterial},
    pipeline::{InstancedWindAffectedPipeline, InstancedWindAffectedPipelineKey},
};
use crate::instancing::pipeline::InstancedComputePipeline;
use crate::prelude::*;
use bevy_asset::Assets;
use bevy_core_pipeline::core_3d::AlphaMask3d;
use bevy_core_pipeline::prepass::{
    DepthPrepass, MotionVectorPrepass, NormalPrepass, OpaqueNoLightmap3dBatchSetKey,
    OpaqueNoLightmap3dBinKey,
};
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemChangeTick;
use bevy_pbr::{MeshPipelineKey, RenderMeshInstances};
use bevy_render::batching::gpu_preprocessing::GpuPreprocessingSupport;
use bevy_render::mesh::allocator::MeshAllocator;
use bevy_render::render_phase::{BinnedRenderPhaseType, ViewBinnedRenderPhases};
use bevy_render::view::Msaa;
use bevy_render::{
    mesh::RenderMesh, render_asset::RenderAssets, render_phase::DrawFunctions, render_resource::*,
    sync_world::MainEntity, view::ExtractedView,
};
use bevy_utils::default;

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
            WindAffectedKey::BILLBOARDING,
            material.options.enable_billboarding,
        );
        key.set(
            WindAffectedKey::EDGE_CORRECTION,
            material.options.edge_correction_factor > 0.,
        );
        key.set(
            WindAffectedKey::WIND_LOW_QUALITY,
            material.options.low_quality,
        );
        key.set(WindAffectedKey::FAST_NORMALS, material.options.fast_normals);
        key.set(
            WindAffectedKey::WIND_AFFECTED,
            material.options.wind_affected,
        );
        key.set(
            WindAffectedKey::STATIC_BEND,
            material.options.static_bend_strength > 0.,
        );
        key.set(WindAffectedKey::DEBUG, material.options.debug);
        key.set(
            WindAffectedKey::ANALYTICAL_NORMALS,
            material.options.analytical_normals,
        );
        key.set(
            WindAffectedKey::CURVE_NORMALS,
            material.options.curve_factor > 0.,
        );
        key.set(WindAffectedKey::POINT_LIGHTS, material.options.point_lights);
        key.set(
            WindAffectedKey::DIRECTIONAL_LIGHTS,
            material.options.directional_lights,
        );

        commands
            .entity(entity)
            .insert(InstancePipelineKey(key.bits()));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_instanced_wind_affected(
    alpha_mask_3d_draw_functions: Res<DrawFunctions<AlphaMask3d>>,
    custom_pipeline: Res<InstancedWindAffectedPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancedWindAffectedPipeline>>,
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
    mesh_allocator: Res<MeshAllocator>,
    gpu_preprocessing_support: Res<GpuPreprocessingSupport>,
    mut alpha_mask_render_phases: ResMut<ViewBinnedRenderPhases<AlphaMask3d>>,
    ticks: SystemChangeTick,
    views: Query<(
        &ExtractedView,
        &Msaa,
        Option<&DepthPrepass>,
        Option<&NormalPrepass>,
        Option<&MotionVectorPrepass>,
    )>,
) {
    let draw_custom = alpha_mask_3d_draw_functions
        .read()
        .id::<DrawInstancedWindAffected>();

    for (view, msaa, depth_prepass, normal_prepass, motion_vector_prepass) in &views {
        let Some(alpha_mask_phase) = alpha_mask_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let mut view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);

        if depth_prepass.is_some() {
            view_key |= MeshPipelineKey::DEPTH_PREPASS;
        }
        if normal_prepass.is_some() {
            view_key |= MeshPipelineKey::NORMAL_PREPASS;
        }
        if motion_vector_prepass.is_some() {
            view_key |= MeshPipelineKey::MOTION_VECTOR_PREPASS;
        }

        for (entity, main_entity, instance_key) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mut key = InstancedWindAffectedPipelineKey {
                mesh_key: view_key
                    | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology()),
                wind_key: WindAffectedKey::from_bits(instance_key.0).unwrap(),
            };

            key.mesh_key |= MeshPipelineKey::MAY_DISCARD;

            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();

            let (vertex_slab, index_slab) = mesh_allocator.mesh_slabs(&mesh_instance.mesh_asset_id);

            alpha_mask_phase.add(
                OpaqueNoLightmap3dBatchSetKey {
                    pipeline,
                    draw_function: draw_custom,
                    material_bind_group_index: None,
                    vertex_slab: vertex_slab.unwrap_or_default(),
                    index_slab,
                },
                OpaqueNoLightmap3dBinKey {
                    asset_id: mesh_instance.mesh_asset_id.into(),
                },
                (entity, *main_entity),
                mesh_instance.current_uniform_index,
                BinnedRenderPhaseType::mesh(
                    mesh_instance.should_batch(),
                    &gpu_preprocessing_support,
                ),
                ticks.this_run(),
            );
        }
    }
}

pub fn queue_instanced_compute_pipeline(
    pipeline_cache: Res<PipelineCache>,
    mut compute_pipeline: ResMut<InstancedComputePipeline>,
) {
    if compute_pipeline.pipeline_id.is_some() {
        return;
    }

    let id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("instanced_compute_pipeline".into()),
        layout: vec![compute_pipeline.layout.clone()],
        push_constant_ranges: vec![],
        shader: compute_pipeline.shader.clone(),
        shader_defs: vec![],
        entry_point: Some("main".into()),
        ..default()
    });

    compute_pipeline.pipeline_id = Some(id);
}
