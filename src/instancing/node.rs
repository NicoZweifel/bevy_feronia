use crate::instancing::{
    components::{GpuDrawIndexedIndirect, InstancedComputeBindGroup, InstancedComputeSourceBuffer},
    pipeline::InstancedComputePipeline,
};
use bevy_ecs::prelude::*;
use bevy_render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
};

#[cfg(feature = "trace")]
use tracing::{error, trace};

enum InstancedComputeNodeState {
    Loading,
    Ready,
}

pub struct InstancedComputeNode {
    state: InstancedComputeNodeState,
    query: QueryState<(
        &'static InstancedComputeSourceBuffer,
        &'static InstancedComputeBindGroup,
        &'static GpuDrawIndexedIndirect,
    )>,
}

impl FromWorld for InstancedComputeNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: InstancedComputeNodeState::Loading,
            query: world.query_filtered(),
        }
    }
}

impl Node for InstancedComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<InstancedComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        if let InstancedComputeNodeState::Loading = self.state {
            if let Some(id) = pipeline.pipeline_id {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(id) {
                    self.state = InstancedComputeNodeState::Ready;
                }
            }
        }

        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let InstancedComputeNodeState::Loading = self.state {
            return Ok(());
        }

        let pipeline_res = world.resource::<InstancedComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        if let Some(id) = pipeline_res.pipeline_id {
            match pipeline_cache.get_compute_pipeline_state(id) {
                CachedPipelineState::Err(_err) => {
                    #[cfg(feature = "trace")]
                    error!("Instanced compute pipeline error: {:?}", _err);
                }
                CachedPipelineState::Queued => {
                    #[cfg(feature = "trace")]
                    trace!("Instanced compute pipeline is still compiling...");
                }
                CachedPipelineState::Ok(_) => {
                    #[cfg(feature = "trace")]
                    trace!("Instanced compute pipeline is ok...");
                }
                CachedPipelineState::Creating(_) => {
                    #[cfg(feature = "trace")]
                    trace!("Compute pipeline is creating...");
                }
            }
        }

        let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_res.pipeline_id.unwrap())
        else {
            return Ok(());
        };

        let mut pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("instanced_gpu_cull_pass"),
                    timestamp_writes: None,
                });

        pass.set_pipeline(pipeline);

        for (source, bind_group, _indirect) in self.query.iter_manual(world) {
            pass.set_bind_group(0, &bind_group.0, &[]);

            let workgroups = (source.count as f32 / 64.0).ceil() as u32;
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        Ok(())
    }
}
