use crate::prelude::*;
use bevy_image::Image;
use bevy_math::{FloatExt, Vec2, Vec3, Vec3Swizzles};
use std::ops::Range;

#[cfg(feature = "trace")]
use tracing::warn;

pub struct HeightMapCpuSampler<'a> {
    image_data: &'a Option<Vec<u8>>,
    image_size: u32,
    world_height_range: Range<f32>,
    total_world_size: f32,
}

impl<'a> HeightMapCpuSampler<'a> {
    pub fn new(image: &'a Image, config: &HeightMapConfig) -> Self {
        Self {
            image_data: &image.data,
            image_size: image.texture_descriptor.size.width,
            world_height_range: config.world_height_range.clone(),
            total_world_size: config.world_size,
        }
    }

    /// Fetches the raw, normalized height value [0.0, 1.0] from the texture data at a given pixel coordinate.
    fn get_normalized_height_at(&self, x: u32, y: u32) -> f32 {
        let Some(data) = self.image_data.as_ref() else {
            return 0.0;
        };
        let byte_index = (y * self.image_size + x) as usize * 4;

        // Each pixel is an R32Float, which is 4 bytes.
        data.get(byte_index..byte_index + 4)
            .and_then(|slice| slice.try_into().ok())
            .map(f32::from_le_bytes)
            .unwrap_or_else(|| {
                #[cfg(feature = "trace")]
                warn!("Failed to read heightmap pixel at ({x}, {y})");
                0.0
            })
    }
}

impl<'a> Sampler for HeightMapCpuSampler<'a> {
    /// Calculates the terrain height at a given world position using bilinear interpolation.
    fn sample(&self, world_pos: Vec3) -> f32 {
        // Convert world coordinates to floating-point pixel coordinates.
        let uv =
            (world_pos.xz() + Vec2::splat(self.total_world_size / 2.0)) / self.total_world_size;
        let pixel_f = uv.clamp(Vec2::ZERO, Vec2::ONE) * (self.image_size - 1) as f32;

        // Get corner coordinates and interpolation weights.
        let corner_i = pixel_f.floor();
        let weight = pixel_f.fract();

        let x0 = corner_i.x as u32;
        let y0 = corner_i.y as u32;
        let x1 = (x0 + 1).min(self.image_size - 1);
        let y1 = (y0 + 1).min(self.image_size - 1);

        //  Fetch the four corner height values.
        let h00 = self.get_normalized_height_at(x0, y0); // Top-left
        let h10 = self.get_normalized_height_at(x1, y0); // Top-right
        let h01 = self.get_normalized_height_at(x0, y1); // Bottom-left
        let h11 = self.get_normalized_height_at(x1, y1); // Bottom-right

        // Interpolate and denormalize.
        let top = h00.lerp(h10, weight.x);
        let bottom = h01.lerp(h11, weight.x);
        let normalized_height = top.lerp(bottom, weight.y);

        self.world_height_range
            .start
            .lerp(self.world_height_range.end, normalized_height)
    }
}
