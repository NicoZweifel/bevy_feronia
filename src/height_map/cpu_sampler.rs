use crate::prelude::*;
use bevy::prelude::*;
use std::ops::Range;

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
            warn!("Attempted to sample from a heightmap with no CPU-side data.");
            return 0.0;
        };

        // Each pixel is an R32Float, which is 4 bytes.
        let byte_index = (y * self.image_size + x) as usize * 4;

        if let Some(pixel_bytes) = data.get(byte_index..byte_index + 4) {
            if let Ok(bytes_array) = pixel_bytes.try_into() {
                f32::from_le_bytes(bytes_array)
            } else {
                warn!("Failed to convert byte slice to array at ({}, {})", x, y);
                0.0
            }
        } else {
            warn!(
                "Failed to read height map pixel at ({}, {}): index out of bounds.",
                x, y
            );
            0.0
        }
    }
}

impl<'a> Sampler for HeightMapCpuSampler<'a> {
    fn sample(&self, world_pos: Vec3) -> f32 {
        // --- 1. Convert World Position to Floating-Point Pixel Coordinates ---
        let center_offset = self.total_world_size / 2.0;
        let uv_x = ((world_pos.x + center_offset) / self.total_world_size).clamp(0.0, 1.0);
        let uv_z = ((world_pos.z + center_offset) / self.total_world_size).clamp(0.0, 1.0);

        let float_x = uv_x * (self.image_size - 1) as f32;
        let float_y = uv_z * (self.image_size - 1) as f32;

        // --- 2. Identify the Bounding Box of the 4 Surrounding Texels ---
        let x0 = float_x.floor() as u32;
        let y0 = float_y.floor() as u32;

        // Clamp coordinates to prevent out-of-bounds access on the texture edges.
        let x1 = (x0 + 1).min(self.image_size - 1);
        let y1 = (y0 + 1).min(self.image_size - 1);

        // --- 3. Calculate Interpolation Weights (Fractional Coordinates) ---
        let tx = float_x.fract();
        let ty = float_y.fract();

        // --- 4. Fetch the Four Corner Normalized Height Values ---
        let h00 = self.get_normalized_height_at(x0, y0); // Top-left
        let h10 = self.get_normalized_height_at(x1, y0); // Top-right
        let h01 = self.get_normalized_height_at(x0, y1); // Bottom-left
        let h11 = self.get_normalized_height_at(x1, y1); // Bottom-right

        // --- 5. Perform Bilinear Interpolation ---
        // First pass: interpolate along the x-axis for top and bottom edges.
        let h_top = h00 * (1.0 - tx) + h10 * tx;
        let h_bottom = h01 * (1.0 - tx) + h11 * tx;

        // Second pass: interpolate along the y-axis using the results from the first pass.
        let normalized_height = h_top * (1.0 - ty) + h_bottom * ty;

        // --- 6. Denormalize the Value Back to World Height ---
        // Uses the height range to convert the [0.0, 1.0] value back to world units.
        let span = self.world_height_range.end - self.world_height_range.start;

        normalized_height * span + self.world_height_range.start
    }
}
