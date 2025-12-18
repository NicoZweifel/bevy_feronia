use bevy_color::palettes::css::WHITE;
use bevy_color::{Color, palettes::tailwind::*};
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ChunkDebugConfig {
    pub aabb_color: Color,
    pub lod_colors: Vec<Color>,
    pub show_aabbs: bool,
}

impl Default for ChunkDebugConfig {
    fn default() -> Self {
        Self {
            lod_colors: vec![
                RED_500.into(),
                ORANGE_500.into(),
                YELLOW_500.into(),
                WHITE.into(),
            ],
            aabb_color: GREEN_500.into(),
            show_aabbs: false,
        }
    }
}
