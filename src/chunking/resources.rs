use bevy::prelude::*;

#[derive(Resource)]
pub struct ChunkDebugConfig {
    pub aabb_color: Color,
    pub lod_colors: Vec<Color>,
}
