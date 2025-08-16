#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_functions::get_world_from_local

#import bevy_pbr::forward_io::Vertex

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_y: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = get_world_from_local(vertex.instance_index);
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);

    out.clip_position = view.clip_from_world * world_position;
    out.world_y = world_position.y;

    return out;
}

struct HeightSettings {
    min_height: f32,
    max_height: f32,
};

@group(3) @binding(0)
var<uniform> settings: HeightSettings;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let span = settings.max_height - settings.min_height;

    if (span <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let normalized_height = (in.world_y - settings.min_height) / span;

    return vec4<f32>(normalized_height, 0.0, 0.0, 1.0);
}
