use crate::prelude::*;

use bevy_image::Image;
use bevy_math::{Affine3A, FloatExt, Vec2, Vec3, Vec3Swizzles};
use bevy_transform::prelude::GlobalTransform;

use std::ops::Range;

#[cfg(feature = "trace")]
use tracing::warn;

pub struct HeightMapCpuSampler<'a> {
    image_data: &'a Option<Vec<u8>>,
    image_size: u32,
    world_height_range: Range<f32>,
    total_world_size: f32,
    world_center: Vec2,
    world_to_local: Affine3A,
}

impl<'a> HeightMapCpuSampler<'a> {
    pub fn new(
        image: &'a Image,
        config: &HeightMapConfig,
        map_transform: &GlobalTransform,
    ) -> Self {
        Self {
            image_data: &image.data,
            image_size: image.texture_descriptor.size.width,
            world_height_range: config.world_height_range.clone(),
            total_world_size: config.world_size,
            world_center: config.world_center,
            world_to_local: map_transform.affine().inverse(),
        }
    }

    fn get_normalized_height_at(&self, x: u32, y: u32) -> f32 {
        let Some(data) = self.image_data.as_ref() else {
            return 0.0;
        };
        let byte_index = (y * self.image_size + x) as usize * 4;

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
        let local_pos = self.world_to_local.transform_point3(world_pos);
        let map_local = local_pos.xz() - self.world_center;

        // Convert world coordinates to floating-point pixel coordinates.
        let uv = (map_local + Vec2::splat(self.total_world_size / 2.0)) / self.total_world_size;

        let pixel = uv.clamp(Vec2::ZERO, Vec2::ONE) * (self.image_size - 1) as f32;

        // Get corner coordinates and interpolation weights.
        let corner = pixel.floor();
        let weight = pixel.fract();

        let x0 = corner.x as u32;
        let y0 = corner.y as u32;
        let x1 = (x0 + 1).min(self.image_size - 1);
        let y1 = (y0 + 1).min(self.image_size - 1);

        // Fetch the four corner height values.
        let h00 = self.get_normalized_height_at(x0, y0); // Top-left
        let h10 = self.get_normalized_height_at(x1, y0); // Top-right
        let h01 = self.get_normalized_height_at(x0, y1); // Bottom-left
        let h11 = self.get_normalized_height_at(x1, y1); // Bottom-right

        // Interpolate and denormalize.
        let top = h00.lerp(h10, weight.x);
        let bottom = h01.lerp(h11, weight.x);
        let normalized_height = top.lerp(bottom, weight.y);

        let local_height = self
            .world_height_range
            .start
            .lerp(self.world_height_range.end, normalized_height);

        let local_height = Vec3::new(local_pos.x, local_height, local_pos.z);

        self.world_to_local
            .inverse()
            .transform_point3(local_height)
            .y
    }
}
