// References (more specific references listed below):
// - https://gpuweb.github.io/gpuweb/wgsl/
// - https://github.com/bevyengine/bevy/blob/b995b4662a9481e8d6fd984b2d2fd02e1c2d1a5d/crates/bevy_pbr/src/render/
//   (Multiple shaders were referenced to figure out the in-build pipeline and how to integrate with the fragment shader)

/* Adapted from examples in https://github.com/bevyengine/bevy/blob/b995b4662a9481e8d6fd984b2d2fd02e1c2d1a5d/assets/shaders/ */
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_functions

/* Binding structures and bindings adapted from
 * https://github.com/bevyengine/bevy/blob/b995b4662a9481e8d6fd984b2d2fd02e1c2d1a5d/crates/bevy_pbr/src/render/mesh.wgsl */
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
}

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

@group(1) @binding(11)
var displacement: texture_2d<f32>;
@group(1) @binding(12)
var displacement_sampler: sampler;
@group(1) @binding(13)
var normal: texture_2d<f32>;
@group(1) @binding(14)
var normal_sampler: sampler;
@group(1) @binding(15)
var<uniform> amplitude: f32;

/* References for the vertex shader:
 * - https://en.wikipedia.org/wiki/Trochoidal_wave
 * - Tessendorf, Jerry. (2001). Simulating Ocean Water. SIG-GRAPH'99 Course Note. */
@vertex
fn vertex(vert_in: VertexIn) -> VertexOut {
    var out: VertexOut;

    let displacement_sample = textureSampleLevel(displacement, displacement_sampler, vert_in.uv, 0.0).rgb - 0.5;
    let normal_sample = textureSampleLevel(normal, normal_sampler, vert_in.uv, 0.0).rgb - 0.5;

    let displaced_pos = vert_in.position + amplitude * displacement_sample;
    out.position = mesh_position_local_to_clip(mesh.model, vec4<f32>(displaced_pos, 1.0));
    out.world_normal = normal_sample;
    out.uv = vert_in.uv;
    out.world_tangent = mesh_tangent_local_to_world(mesh.model, vert_in.tangent);
    return out;
}
