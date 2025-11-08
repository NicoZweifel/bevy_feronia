use bevy::render::render_resource::BindGroup;
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::Buffer},
};
use bytemuck::{Pod, Zeroable};

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

#[derive(Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub index: u32,
    pub _padding: [u32; 3],
}

#[derive(Component, Clone)]
pub struct InstanceMaterialData {
    pub instances: Vec<InstanceData>,
    pub color: [f32; 4],
    pub visibility_range: [f32; 4],
    pub static_bend_strength: f32,
    pub curve_factor: f32,
}

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

#[derive(Component)]
pub struct GpuDrawIndexedIndirect {
    pub buffer: Buffer,
    pub offset: u64,
}

#[derive(Clone, Copy, Pod, Zeroable, Default)]
#[repr(C)]
pub struct InstanceUniforms {
    pub color: [f32; 4],
    pub visibility_range: [f32; 4],
    pub static_bend_strength: f32,
    pub curve_factor: f32,
    pub _padding: [f32; 3],
}

#[derive(Component)]
pub struct InstanceUniformBuffer {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}
