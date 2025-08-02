use crate::prelude::Sampler;
use bevy::image::Image;
use bevy::math::Vec3;

pub struct DensityMapSampler<'a> {
    pub image_data: &'a Option<Vec<u8>>,
    pub image_size: u32,
    pub total_world_size: f32,
    pub center_offset: f32,
}

impl<'a> DensityMapSampler<'a> {
    pub fn new(image: &'a Image, total_world_size: f32) -> Self {
        Self {
            image_data: &image.data,
            image_size: image.texture_descriptor.size.width,
            total_world_size,
            center_offset: total_world_size / 2.0,
        }
    }
}

impl<'a> Sampler for DensityMapSampler<'a> {
    fn sample(&self, world_pos: Vec3) -> f32 {
        let uv_x = ((world_pos.x + self.center_offset) / self.total_world_size).clamp(0.0, 1.0);
        let uv_y = ((world_pos.z + self.center_offset) / self.total_world_size).clamp(0.0, 1.0);
        let pixel_x = (uv_x * (self.image_size - 1) as f32).round() as u32;
        let pixel_y = (uv_y * (self.image_size - 1) as f32).round() as u32;
        let pixel_index = (pixel_y * self.image_size + pixel_x) as usize;

        let sampled_byte = self
            .image_data
            .as_ref()
            .unwrap()
            .get(pixel_index)
            .copied()
            .unwrap_or(0);

        sampled_byte as f32 / 255.0
    }
}
