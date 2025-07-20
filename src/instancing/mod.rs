use bevy::{
    asset::embedded_asset,
    core_pipeline::core_3d::Transparent3d,
    pbr::*,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_phase::AddRenderCommand,
        render_resource::*,
        Render, RenderApp, RenderSystems,
    },
};
use bevy::render::render_asset::RenderAssetPlugin;

mod components;
mod draw;
mod material;
mod pipeline;
mod systems;

pub mod prelude {
    pub use super::{
        components::{InstanceData, InstanceMaterialData},
        material::{InstancedWindAffectedMaterial, InstancedWindAffectedMeshMaterial},
        InstancedWindAffectedPlugin,
    };
}

use components::{InstanceMaterialData, InstancePipelineKey};
use draw::DrawCustom;
use material::{
    InstancedWindAffectedMaterial, InstancedWindAffectedMeshMaterial,
    PreparedInstancedWindAffectedMaterial,
};
use pipeline::CustomPipeline;
use systems::{add_instance_key_component, prepare_instance_buffers, queue_custom};
use crate::plugin::WindMaterialPlugin;

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
        .add_systems(Update, add_instance_key_component);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSystems::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSystems::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<CustomPipeline>();
    }
}