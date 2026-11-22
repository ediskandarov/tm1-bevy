import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear};
import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var nearest: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas, nearest, in.uv);
    if texel.a == 0.0 {
        return vec4(0.0);
    }
    // STP texels add their light to the destination. Ordinary texels replace it.
    let light = srgb_to_linear(linear_to_srgb(texel.rgb) * in.color.rgb);
    return vec4(light, select(0.0, 1.0, texel.a > 0.75));
}
