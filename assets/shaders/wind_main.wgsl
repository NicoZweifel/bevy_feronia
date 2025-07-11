#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::{get_model_matrix, get_world_from_local}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::{
    pbr_fragment::{pbr_input_from_standard_material, pbr_material_from_standard_material},
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing}
}
#import bevy_pbr::forward_io::{Vertex, VertexOutput, FragmentOutput}

#import "./shaders/wind.wgsl"::Wind
#import "shaders/wind_displace.wgsl"::{DisplacedVertex, displace_vertex_and_calc_normal}
#import bevy_pbr::mesh_view_bindings::globals


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
        vertex.normal,
        view.world_position.xyz
    );

    out.position = position_world_to_clip(displaced.world_position.xyz);
    out.world_position = displaced.world_position;
    out.world_normal = displaced.world_normal;

    out.uv = vertex.uv;
    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
