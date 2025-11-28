use bevy_color::Color;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ChunkDebugConfig {
    pub aabb_color: Color,
    pub lod_colors: Vec<Color>,
}
