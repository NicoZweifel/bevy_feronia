use super::prepare::*;
use super::{
    components::GpuCull,
    draw::DrawInstancedWindAffected,
    node::InstancedComputeNode,
    pipeline::{InstancedComputePipeline, InstancedWindAffectedPipeline},
    systems::*,
};
use crate::core::events::SpawnScatterAssets;
use crate::prelude::*;

use crate::instancing::resources::GrassBufferCache;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_asset::{AssetApp, embedded_asset};
use bevy_core_pipeline::core_3d::AlphaMask3d;
use bevy_ecs::prelude::*;
use bevy_render::graph::CameraDriverLabel;
use bevy_render::{
    Render, RenderApp, RenderSystems,
    extract_component::ExtractComponentPlugin,
    render_asset::RenderAssetPlugin,
    render_graph::{RenderGraph, RenderLabel},
    render_phase::AddRenderCommand,
    render_resource::SpecializedMeshPipelines,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct InstancedComputeLabel;

pub struct InstancedWindAffectedPlugin;

impl Plugin for InstancedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "instanced.wgsl");
        embedded_asset!(app, "compute.wgsl");

        app.init_asset::<InstancedWindAffectedMaterial>()
            .add_message::<SpawnScatterAssets<InstancedWindAffectedMaterial>>();

        app.add_plugins((
            ScatterMaterialPlugin::<InstancedWindAffectedMaterial>::default(),
            ExtractComponentPlugin::<InstancePipelineKey>::default(),
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractComponentPlugin::<InstancedWindAffectedMeshMaterial>::default(),
            ExtractComponentPlugin::<GpuCull>::default(),
            ExtractComponentPlugin::<Center>::default(),
            ExtractComponentPlugin::<CullLodDensity>::default(),
            RenderAssetPlugin::<PreparedInstancedWindAffectedMaterial>::default(),
        ))
        .add_systems(PostUpdate, add_instance_key_component);

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_command::<AlphaMask3d, DrawInstancedWindAffected>()
            .init_resource::<GrassBufferCache>()
            .init_resource::<SpecializedMeshPipelines<InstancedWindAffectedPipeline>>()
            .add_systems(
                Render,
                (
                    (
                        queue_instanced_wind_affected,
                        queue_instanced_compute_pipeline,
                    )
                        .in_set(RenderSystems::QueueMeshes),
                    (
                        // CPU
                        prepare_instance_buffer,
                        prepare_indirect_draw_buffer.after(queue_instanced_wind_affected),
                        // GPU
                        prepare_instanced_compute_source,
                        prepare_instanced_compute_output,
                        prepare_instanced_compute_indirect,
                        prepare_lod_buffer,
                        prepare_instanced_compute_bind_group.after(prepare_global_cull_buffer),
                        prepare_reset_indirect_buffer.after(prepare_instanced_compute_indirect),
                        // Common
                        prepare_global_cull_buffer,
                        prepare_instance_uniform_buffer,
                    )
                        .in_set(RenderSystems::PrepareResources),
                ),
            );

        let compute_node = InstancedComputeNode::from_world(render_app.world_mut());

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        render_graph.add_node(InstancedComputeLabel, compute_node);
        render_graph.add_node_edge(InstancedComputeLabel, CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<InstancedWindAffectedPipeline>()
            .init_resource::<InstancedComputePipeline>();
    }
}

pub struct InstancedWindAffectedScatterPlugin;

impl Plugin for InstancedWindAffectedScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedWindAffectedPlugin,
            ScatterAssetsPlugin::<InstancedWindAffectedMaterial>::new(),
            ScatterAssetPlugin::<InstancedWindAffectedMaterial>::new(),
        ));
    }
}
