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

pub struct TextureBlendPlugin;
impl Plugin for TextureBlendPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "texture_blend.wesl");
        embedded_asset!(app, "arena_mask.wesl");
        app.add_plugins((
            Material2dPlugin::<TextureBlend>::default(),
            Material2dPlugin::<ArenaMask>::default(),
        ));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ArenaMask {
    #[texture(0)]
    #[sampler(1)]
    pub atlas: Handle<Image>,
}

impl Material2d for ArenaMask {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Mask(0.5)
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://tm1_bevy/arena_mask.wesl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(bool)]
pub struct TextureBlend {
    #[texture(0)]
    #[sampler(1)]
    pub atlas: Handle<Image>,
    #[uniform(2)]
    pub mode: u32,
}
impl From<&TextureBlend> for bool {
    fn from(material: &TextureBlend) -> bool {
        material.mode == 2
    }
}

impl Material2d for TextureBlend {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://tm1_bevy/texture_blend.wesl".into()
    }
    fn specialize(
        _: &Material2dPipeline,
        desc: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
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
                    operation: if key.bind_group_data {
                        BlendOperation::ReverseSubtract
                    } else {
                        BlendOperation::Add
                    },
                },
                alpha: BlendComponent::OVER,
            });
        }
        Ok(())
    }
}
