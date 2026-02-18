use crate::prelude::HeightMapMaterial;

use bevy_asset::Handle;
use bevy_camera::visibility::RenderLayers;
use bevy_color::{Color, palettes::tailwind::GREEN_500};
use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_math::Vec2;
use bevy_reflect::Reflect;
use derive_more::{From, Into};

use std::ops::Range;


#[derive(Resource, Clone, Reflect, Debug)]
#[reflect(Resource, Clone, Debug)]
pub struct HeightMapConfig {
    pub world_size: f32,
    pub world_center: Vec2,
    pub render_layer: RenderLayers,
    pub world_height_range: Range<f32>,
}

#[derive(Resource, Clone, Reflect, Debug, From, Into, Copy)]
#[reflect(Resource, Clone, Debug)]
pub struct HeightMapDebugConfig(pub Color);

impl HeightMapDebugConfig {
    pub fn new(color: impl Into<Color>) -> Self {
        Self(color.into())
    }
}

impl Default for HeightMapDebugConfig {
    fn default() -> Self {
        Self::new(GREEN_500)
    }
}

impl Default for HeightMapConfig {
    fn default() -> Self {
        Self {
            world_height_range: -28. ..100.,
            world_size: 8.0 * 8.0 * 4.0,
            render_layer: RenderLayers::default(),
            world_center: Vec2::ZERO,
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HeightMapTexture(pub Handle<Image>);

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct HeightMap(pub Handle<Image>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HeightMapMaterialHandle(pub Handle<HeightMapMaterial>);
