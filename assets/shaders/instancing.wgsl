#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings::globals

#import "./shaders/wind.wgsl"::Wind
#import "shaders/wind_displace.wgsl"::{DisplacedVertex, SampledNoise, InstanceInfo,  displace_vertex_and_calc_normal}

@group(2) @binding(50) var<uniform> wind: Wind;
@group(2) @binding(51) var noise_texture: texture_2d<f32>;
@group(2) @binding(52) var noise_texture_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(8) i_pos_scale: vec4<f32>,
    @location(9) i_color: vec4<f32>,
    @location(10) i_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // --- INSTANCE ---
    var instance: InstanceInfo;

    instance.world_from_local = mat4x4<f32>(
        vec4<f32>(vertex.i_pos_scale.w, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, vertex.i_pos_scale.w, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, vertex.i_pos_scale.w, 0.0),
        vec4<f32>(vertex.i_pos_scale.xyz, 1.0)
    );

    instance.instance_position = instance.world_from_local[3];
    instance.wrapped_time = globals.time % 1000.0;
    instance.instance_index = vertex.i_index;

    // --- TEXTURE SAMPLING ---
    let dist_to_camera = distance(instance.instance_position.xyz, view.world_position.xyz);
    let lod_fade = smoothstep(wind.lod_threshold * 2.0, wind.lod_threshold, dist_to_camera);

    var noise: SampledNoise;
    noise.micro_noise = 0.0;
    noise.phase_noise = vec2<f32>(0.0);

    let macro_coord = instance.instance_position.xz * wind.noise_scale + instance.wrapped_time * wind.scroll_speed * wind.direction;
    noise.macro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, macro_coord, 0.0).r;


    if (lod_fade > 0.0) {
        let micro_coord = instance.instance_position.xz * wind.micro_noise_scale + instance.wrapped_time * wind.micro_scroll_speed;
        noise.micro_noise = textureSampleLevel(noise_texture, noise_texture_sampler, micro_coord, 0.0).r;

        let texture_dimension = 512.0;
        let phase_coord_x = f32(instance.instance_index % u32(texture_dimension)) / texture_dimension;
        let phase_coord_y = f32(instance.instance_index / u32(texture_dimension)) / texture_dimension;
        let phase_coord = vec2<f32>(phase_coord_x, phase_coord_y);
        let phase_sample = textureSampleLevel(noise_texture, noise_texture_sampler, phase_coord, 0.0);
        noise.phase_noise = vec2(phase_sample.g, phase_sample.b);
    }

    // --- DISPLACEMENT ---
    let displaced = displace_vertex_and_calc_normal(
        wind,
        noise,
        vertex.position,
        instance,
        dist_to_camera,
        vertex.normal,
        vertex.uv
    );

    out.clip_position = view.clip_from_world * displaced.world_position;

    out.color = vertex.i_color;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
