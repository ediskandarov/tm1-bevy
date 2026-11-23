import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear};
import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var nearest: sampler;

// Vertex alpha identifies the packet's PS1 blend mode. Atlas alpha encodes STP.
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let mode = u32(round(in.color.a));
    let texel = textureSample(atlas, nearest, in.uv);
    let ordinary = texel.a > 0.75;
    if texel.a == 0.0 || (mode == 2u && ordinary) || (mode == 5u && !ordinary) {
        discard;
    }
    if mode == 0u {
        return vec4(texel.rgb * in.color.rgb * texel.a, texel.a);
    }
    if mode >= 4u {
        return vec4(texel.rgb * in.color.rgb, 1.0);
    }
    let foreground = min(linear_to_srgb(texel.rgb) * in.color.rgb, vec3(1.0));
    let light = srgb_to_linear(foreground * select(1.0, 0.25, mode == 3u && !ordinary));
    return vec4(light, select(0.0, 1.0, ordinary));
}
