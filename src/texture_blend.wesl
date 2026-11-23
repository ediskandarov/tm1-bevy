import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear};
import bevy_sprite_render::mesh2d::{
    functions as mesh_functions,
    vertex_input::{Vertex, decompress_vertex}
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var nearest: sampler;

struct PacketVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(linear) uv: vec2<f32>,
    @location(1) @interpolate(linear) color: vec3<f32>,
    @location(2) @interpolate(flat) mode: u32,
}

// PS1 packets contain screen coordinates and have no reciprocal depth for UV or RGB.
@vertex
fn vertex(input: Vertex) -> PacketVertex {
    let v = decompress_vertex(input, input.instance_index);
    let flags = u32(round(v.color.a));
    if (flags & 16u) != 0u {
        return PacketVertex(vec4(v.position, 1.0), v.uv, v.color.rgb, flags & 15u);
    }
    let transform = mesh_functions::get_world_from_local(input.instance_index);
    let world = mesh_functions::mesh2d_position_local_to_world(transform, vec4(v.position, 1.0));
    return PacketVertex(mesh_functions::mesh2d_position_world_to_clip(world),
                        v.uv, v.color.rgb, flags);
}

// Vertex alpha identifies the packet's PS1 blend mode. Atlas alpha encodes STP.
@fragment
fn fragment(in: PacketVertex) -> @location(0) vec4<f32> {
    let mode = in.mode;
    let texel = textureSample(atlas, nearest, in.uv);
    let ordinary = texel.a > 0.75;
    if texel.a == 0.0 || (mode == 2u && ordinary) || (mode == 5u && !ordinary) {
        discard;
    }
    // Recover the five-bit texel from the replicated RGB555 atlas bytes.
    // Texture modulation uses 0x80 for unity. Untextured packets carry RGB888.
    let tex5 = floor(round(linear_to_srgb(texel.rgb) * 255.0) / 8.0);
    let command = round(clamp(in.color.rgb * 128.0, vec3(0.0), vec3(255.0)));
    var source = min(floor(tex5 * command / 128.0), vec3(31.0));
    if mode == 6u || mode == 7u {
        source = floor(round(clamp(in.color.rgb, vec3(0.0), vec3(1.0)) * 255.0) / 8.0);
    }
    if mode == 3u && !ordinary {
        source = floor(source / 4.0);
    }
    let light = srgb_to_linear((source * 8.0 + floor(source / 4.0)) / 255.0);
    if mode == 0u || mode == 7u {
        let coverage = select(0.5, 1.0, mode == 0u && ordinary);
        return vec4(light * coverage, coverage);
    }
    return vec4(light, select(0.0, 1.0, ordinary || mode == 4u || mode == 6u));
}
