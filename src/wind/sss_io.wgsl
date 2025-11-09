#define_import_path bevy_feronia::sss_io

struct VertexOutput {
    // Standard Bevy PBR fields
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) uv_b: vec2<f32>,

    @location(4) world_tangent: vec4<f32>,

    @location(6) instance_index: u32,
    @location(7) visibility_range_dither : i32,

    /// SSS fields
    @location(8) thinness_factor: f32,
};