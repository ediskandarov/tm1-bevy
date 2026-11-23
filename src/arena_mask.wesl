import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::view};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var atlas: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var nearest: sampler;

struct Output {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> Output {
    let texel = textureSample(atlas, nearest, in.uv);
    if texel.a < 0.5 {
        discard;
    }
    let near = view.clip_from_view[3].z;
    let bucket = round(in.color.a);
    let depth = near / max(near + 0.01, bucket * 8.0);
    return Output(vec4(texel.rgb * in.color.rgb, 1.0), depth);
}
