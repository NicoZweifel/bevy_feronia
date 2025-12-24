use crate::prelude::Sampler;
use bevy_image::Image;
use bevy_math::{UVec3, Vec3};
use bevy_render::render_resource::TextureFormat;

/// Texture sampler for the density map image.
///
/// It samples red channel only and converts it to [0.0, 1.0] range.
pub struct DensityMapSampler<'a> {
    pub image: &'a Image,
    pub total_world_size: Vec3,
    pub center_offset: Vec3,
}

impl<'a> DensityMapSampler<'a> {
    pub fn new(image: &'a Image, total_world_size: Vec3) -> Self {
        Self {
            image,
            total_world_size,
            center_offset: total_world_size / 2.0,
        }
    }
}

impl<'a> Sampler for DensityMapSampler<'a> {
    fn sample(&self, world_pos: Vec3) -> f32 {
        let uv_x = ((world_pos.x + self.center_offset.x) / self.total_world_size.x).clamp(0.0, 1.0);
        let uv_y = ((world_pos.z + self.center_offset.z) / self.total_world_size.z).clamp(0.0, 1.0);
        let pixel_x = (uv_x * (self.image.width() - 1) as f32).round() as u32;
        let pixel_y = (uv_y * (self.image.height() - 1) as f32).round() as u32;

        let Some(bytes) = self.image.pixel_bytes(UVec3::new(pixel_x, pixel_y, 0)) else {
            return 0.0;
        };

        match self.image.texture_descriptor.format {
            TextureFormat::Rg8Unorm
            | TextureFormat::Rg8Uint
            | TextureFormat::R8Unorm
            | TextureFormat::R8Uint
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8Uint => bytes[0] as f32 / u8::MAX as f32,
            TextureFormat::Bgra8UnormSrgb | TextureFormat::Bgra8Unorm => {
                bytes[2] as f32 / u8::MAX as f32
            }
            TextureFormat::Rg32Float | TextureFormat::R32Float | TextureFormat::Rgba32Float => {
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            }
            TextureFormat::Rg16Float | TextureFormat::R16Float | TextureFormat::Rgba16Float => {
                f32::from_le_bytes([0, 0, bytes[0], bytes[1]])
            }
            TextureFormat::Rg16Unorm
            | TextureFormat::Rg16Uint
            | TextureFormat::R16Unorm
            | TextureFormat::R16Uint
            | TextureFormat::Rgba16Unorm
            | TextureFormat::Rgba16Uint => {
                ((u16::from_le_bytes([bytes[0], bytes[1]])) as f64 / u16::MAX as f64) as f32
            }
            TextureFormat::Rg32Uint | TextureFormat::R32Uint | TextureFormat::Rgba32Uint => {
                (u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
                    / u32::MAX as f64) as f32
            }
            _ => 0.0,
        }
    }
}
