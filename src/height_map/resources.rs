use crate::prelude::HeightMapMaterial;
use bevy_asset::Handle;
use bevy_camera::visibility::RenderLayers;
use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_reflect::Reflect;
use std::ops::Range;

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
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

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HeightMapTexture(pub Handle<Image>);

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct HeightMap(pub Handle<Image>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HeightMapMaterialHandle(pub Handle<HeightMapMaterial>);
