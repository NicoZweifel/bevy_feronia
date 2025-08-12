use crate::prelude::HeightMapMaterial;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use std::ops::Range;

#[derive(Resource)]
pub struct HeightMapConfig {
    pub world_size: f32,
    pub render_layer: RenderLayers,
    pub world_height_range: Range<f32>,
}

impl Default for HeightMapConfig {
    fn default() -> Self {
        Self {
            world_height_range: -28. ..100.,
            world_size: 8.0 * 8.0 * 4.0,
            render_layer: RenderLayers::layer(1),
        }
    }
}

#[derive(Resource)]
pub struct HeightMapTexture(pub Handle<Image>);

#[derive(Resource, Default, Deref)]
pub struct HeightMap(pub Handle<Image>);

#[derive(Resource)]
pub struct HeightMapMaterialHandle(pub Handle<HeightMapMaterial>);
