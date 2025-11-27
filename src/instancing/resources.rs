use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;
use bevy_math::prelude::*;
use bevy_platform::collections::HashMap;
use bevy_render::render_resource::{Buffer, ShaderType};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, Default, ShaderType)]
#[repr(C)]
pub(super) struct CameraCullData {
    pub view_pos: Vec4,
}

#[derive(Clone, Copy, Pod, Zeroable, Default, ShaderType)]
#[repr(C)]
pub(super) struct LodCullData {
    pub visibility_range: Vec4,
}

#[derive(Resource)]
pub(super) struct GlobalCullBuffer {
    pub buffer: Buffer,
}

#[derive(Resource, Default)]
pub(super) struct GrassBufferCache {
    pub buffers: HashMap<Entity, Buffer>,
}
