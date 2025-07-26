use super::prepare::prepare_instance_buffer;
use super::{draw::DrawCustom, pipeline::CustomPipeline, systems::*};
use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::core_pipeline::core_3d::Transparent3d;
use bevy::prelude::*;
use bevy::render::{
    extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin, render_phase::AddRenderCommand, render_resource::SpecializedMeshPipelines,
    Render, RenderApp,
    RenderSystems,
};

pub struct InstancedWindAffectedPlugin;

impl Plugin for InstancedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "instancing.wgsl");

        app.init_asset::<InstancedWindAffectedMaterial>();
        app.add_plugins((
            WindMaterialPlugin::<StandardMaterial, InstancedWindAffectedMaterial>::default(),
            ExtractComponentPlugin::<InstancePipelineKey>::default(),
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractComponentPlugin::<InstancedWindAffectedMeshMaterial>::default(),
            RenderAssetPlugin::<PreparedInstancedWindAffectedMaterial>::default(),
        ))
        .add_systems(PostUpdate, add_instance_key_component);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSystems::QueueMeshes),
                    prepare_instance_buffer.in_set(RenderSystems::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}
