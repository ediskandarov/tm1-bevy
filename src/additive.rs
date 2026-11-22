use bevy::{
    asset::embedded_asset,
    mesh::MeshVertexBufferLayoutRef,
    prelude::*,
    render::render_resource::{
        AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
        RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin},
};

pub struct AdditivePlugin;
impl Plugin for AdditivePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "additive.wesl");
        app.add_plugins(Material2dPlugin::<Additive>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct Additive {
    #[texture(0)]
    #[sampler(1)]
    pub atlas: Handle<Image>,
    #[uniform(2)]
    pub foreground_scale: f32,
}
impl Material2d for Additive {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://tm1_bevy/additive.wesl".into()
    }
    fn specialize(
        _: &Material2dPipeline,
        desc: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(target) = desc
            .fragment
            .as_mut()
            .and_then(|f| f.targets.first_mut())
            .and_then(Option::as_mut)
        {
            target.blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
        }
        Ok(())
    }
}
