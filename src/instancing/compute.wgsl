#import bevy_pbr::utils::rand_f

struct InstanceData {
    pos_and_scale: vec4<f32>,
    index: u32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
}

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

struct CameraCullData {
    view_pos: vec4<f32>,
}

struct LodCullData {
    visibility_range: vec4<f32>,
    world_from_local: mat4x4<f32>,
}

@group(0) @binding(0) var<storage, read> source_buffer: array<InstanceData>;
@group(0) @binding(1) var<storage, read_write> instance_buffer: array<InstanceData>;
@group(0) @binding(2) var<storage, read_write> indirect_args: DrawIndexedIndirectArgs;
@group(0) @binding(3) var<uniform> camera: CameraCullData;
@group(0) @binding(4) var<uniform> lod_data: LodCullData;


fn hash_noise(index: u32) -> f32 {
    var state = index;
    return rand_f(&state);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= arrayLength(&source_buffer)) { return; }

    let instance = source_buffer[i];
    let local_pos = vec4<f32>(instance.pos_and_scale.xyz, 1.0);
    let world_pos = lod_data.world_from_local * local_pos;

    let dist = distance(world_pos.xyz, camera.view_pos.xyz);

    if (dist < lod_data.visibility_range.x || dist > lod_data.visibility_range.w) {
        return;
    }

    let write_index = atomicAdd(&indirect_args.instance_count, 1u);

    instance_buffer[write_index] = instance;
}