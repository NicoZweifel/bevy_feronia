use super::prepare::{
    prepare_indirect_draw_buffer, prepare_instance_buffer, prepare_instance_uniform_buffer,
};
use super::{draw::DrawInstancedWindAffected, pipeline::InstancedWindAffectedPipeline, systems::*};
use crate::prelude::*;
use bevy::asset::embedded_asset;
use bevy::core_pipeline::core_3d::AlphaMask3d;
use bevy::prelude::*;
use bevy::render::{
    Render, RenderApp, RenderSystems, extract_component::ExtractComponentPlugin,
    render_asset::RenderAssetPlugin, render_phase::AddRenderCommand,
    render_resource::SpecializedMeshPipelines,
};

pub struct InstancedWindAffectedPlugin;

impl Plugin for InstancedWindAffectedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "instancing.wgsl");

        app.init_asset::<InstancedWindAffectedMaterial>()
            .add_message::<SpawnProtoTypes<InstancedWindAffectedMaterial>>();
        app.add_plugins((
            ScatterMaterialPlugin::<InstancedWindAffectedMaterial>::default(),
            ExtractComponentPlugin::<InstancePipelineKey>::default(),
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractComponentPlugin::<InstancedWindAffectedMeshMaterial>::default(),
            RenderAssetPlugin::<PreparedInstancedWindAffectedMaterial>::default(),
        ))
        .add_systems(
            Update,
            InstancedWindAffectedMaterial::spawn
                .run_if(resource_exists::<Assets<ScatterAsset<ExtendedWindAffectedMaterial>>>),
        )
        .add_systems(PostUpdate, add_instance_key_component);

        app.sub_app_mut(RenderApp)
            .add_render_command::<AlphaMask3d, DrawInstancedWindAffected>()
            .init_resource::<SpecializedMeshPipelines<InstancedWindAffectedPipeline>>()
            .add_systems(
                Render,
                (
                    (
                        queue_instanced_wind_affected,
                        prepare_indirect_draw_buffer.after(queue_instanced_wind_affected),
                    )
                        .in_set(RenderSystems::QueueMeshes),
                    (prepare_instance_buffer, prepare_instance_uniform_buffer)
                        .in_set(RenderSystems::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<InstancedWindAffectedPipeline>();
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
