#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::{get_model_matrix, get_world_from_local}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::prepass_io::{Vertex, VertexOutput}
#import bevy_pbr::prepass_bindings::globals

#import "shaders/wind.wgsl"::Wind
#import "shaders/wind_displace.wgsl"::{DisplacedVertex, displace_vertex_and_calc_normal}
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;
@group(2) @binding(100) var<uniform> wind: Wind;
@group(2) @binding(101) var noise_texture: texture_2d<f32>;
@group(2) @binding(102) var noise_sampler: sampler;


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let displaced = displace_vertex_and_calc_normal(
        globals.time,
        wind,
        noise_texture,
        noise_sampler,
        vertex.position,
        vertex.instance_index,
        view.world_position.xyz
    );

    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;

    #ifdef VERTEX_NORMALS
        out.world_normal = displaced.world_normal;
    #endif

    return out;
}
