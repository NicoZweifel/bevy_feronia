use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::Buffer},
};
use bytemuck::{Pod, Zeroable};

/// Component for pipeline specialization, added in the main world.
#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub(crate) struct InstancePipelineKey(pub u64);

impl ExtractComponent for InstancePipelineKey {
    type QueryData = &'static InstancePipelineKey;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(*item)
    }
}

/// The data for a single instance.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
    pub index: u32,
}

/// A component holding the instance data for a mesh.
#[derive(Component, Deref)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

/// The GPU buffer for Instance data, added in the render world.
#[derive(Component)]
pub(crate) struct InstanceBuffer {
    pub(crate) buffer: Buffer,
    pub(crate) length: usize,
}