use crate::prelude::*;
use bevy::image::Image;
use bevy::math::Vec3;

pub struct HeightMapCpuSampler<'a> {
    image_data: &'a Option<Vec<u8>>,
    image_size: u32,
    total_world_size: f32,
}

impl<'a> HeightMapCpuSampler<'a> {
    pub fn new(image: &'a Image, total_world_size: f32) -> Self {
        Self {
            image_data: &image.data,
            image_size: image.texture_descriptor.size.width,
            total_world_size,
        }
    }
}

impl<'a> Sampler for HeightMapCpuSampler<'a> {
    fn sample(&self, world_pos: Vec3) -> f32 {
        let center_offset = self.total_world_size / 2.0;
        let uv_x = ((world_pos.x + center_offset) / self.total_world_size).clamp(0.0, 1.0);
        let uv_z = ((world_pos.z + center_offset) / self.total_world_size).clamp(0.0, 1.0);

        let pixel_x = (uv_x * (self.image_size - 1) as f32).round() as u32;
        let pixel_y = (uv_z * (self.image_size - 1) as f32).round() as u32;

        let byte_index = (pixel_y * self.image_size + pixel_x) as usize * 4;

        if let Some(pixel_bytes) = self
            .image_data
            .as_ref()
            .unwrap()
            .get(byte_index..byte_index + 4)
        {
            // TODO
            return (f32::from_le_bytes(pixel_bytes.try_into().unwrap()) / 0.01) - 32.;
        }

        println!(
            "Failed to read height map pixel at ({}, {})",
            pixel_x, pixel_y
        );

        0.0
    }
}
