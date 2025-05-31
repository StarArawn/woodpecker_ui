use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{sampler, texture_2d};
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
    CommandEncoderDescriptor, FragmentState, PipelineCache, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, VertexState,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy_vello::render::VelloRenderer;
use bevy_vello::vello::peniko;
use bevy_vello::vello::wgpu::{
    Color, ColorTargetState, ColorWrites, MultisampleState, Operations, Origin3d, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, TexelCopyTextureInfoBase, TextureSampleType,
};

pub struct ConvertRenderTargetPlugin;

impl Plugin for ConvertRenderTargetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderTargetImages>()
            .add_plugins(ExtractResourcePlugin::<RenderTargetImages>::default())
            .add_systems(Update, create_shaders);

        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<ConvertPipeline>().add_systems(
            Render,
            (
                prepare_bind_groups.in_set(RenderSet::Queue),
                render.in_set(RenderSet::Queue).after(prepare_bind_groups),
                override_images.in_set(RenderSet::Queue).after(render),
            ),
        );
    }
}

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct RenderTargetImages {
    pub images: HashMap<Handle<Image>, Handle<Image>>,
    pub vello_images: HashMap<Handle<Image>, peniko::Image>,
    shaders: HashMap<TextureFormat, Handle<Shader>>,
}

fn create_shaders(
    mut render_target_images: ResMut<RenderTargetImages>,
    images: Res<Assets<Image>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let formats = render_target_images
        .images
        .iter()
        .filter_map(|(image, _)| {
            images
                .get(image)
                .map(|image| image.texture_descriptor.format)
        })
        .collect::<Vec<_>>();
    for format in formats {
        if !render_target_images.shaders.contains_key(&format) {
            let shader_handle = shaders.add(Shader::from_wgsl(
                r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,

) -> VertexOutput {
    var out: VertexOutput;

    // hacky way to draw a large triangle
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        0.0,
        1.0
    );

    out.position = pos;
    return out;
}

@group(0) @binding(0) var in_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) ->  @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(in_tex));
    var uv = (in.position.xy / resolution.xy);
    let color = textureSample(in_tex, tex_sampler, uv);
    return color;
}
                "#,
                "",
            ));
            render_target_images.shaders.insert(format, shader_handle);
        }
    }
}

#[derive(Resource, Default)]
pub struct ConvertPipeline {
    pipeline: HashMap<TextureFormat, (CachedRenderPipelineId, BindGroupLayout)>,
}

#[derive(Resource, Default)]
pub struct ConvertBindGroups {
    bind_group: HashMap<Handle<Image>, BindGroup>,
}

fn prepare_bind_groups(
    mut commands: Commands,
    mut pipeline: ResMut<ConvertPipeline>,
    pipeline_cache: ResMut<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_target_images: Res<RenderTargetImages>,
    render_device: Res<RenderDevice>,
) {
    let mut bind_groups = ConvertBindGroups::default();
    for (render_target, _) in render_target_images.images.iter() {
        let Some(org_image) = gpu_images.get(render_target) else {
            continue;
        };

        if !pipeline.pipeline.contains_key(&org_image.texture_format) {
            let Some(shader) = render_target_images.shaders.get(&org_image.texture_format) else {
                continue;
            };

            let texture_bind_group_layout = render_device.create_bind_group_layout(
                "texture converter bindgroup",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::FRAGMENT,
                    (
                        texture_2d(TextureSampleType::Float { filterable: true }),
                        sampler(bevy_vello::vello::wgpu::SamplerBindingType::Filtering),
                    ),
                ),
            );

            pipeline.pipeline.insert(
                org_image.texture_format,
                (
                    pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
                        label: None,
                        layout: vec![texture_bind_group_layout.clone()],
                        push_constant_ranges: Vec::new(),
                        vertex: VertexState {
                            shader: shader.clone(),
                            shader_defs: vec![],
                            entry_point: "vertex".into(),
                            buffers: vec![],
                        },
                        primitive: PrimitiveState {
                            topology: bevy::render::mesh::PrimitiveTopology::TriangleList,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: MultisampleState::default(),
                        fragment: Some(FragmentState {
                            shader: shader.clone(),
                            shader_defs: vec![],
                            entry_point: "fragment".into(),
                            targets: vec![Some(ColorTargetState {
                                format: TextureFormat::Rgba8Unorm,
                                blend: None,
                                write_mask: ColorWrites::ALL,
                            })],
                        }),
                        zero_initialize_workgroup_memory: false,
                    }),
                    texture_bind_group_layout,
                ),
            );
        }

        // Create bind groups
        let (_, bg_layout) = pipeline.pipeline.get(&org_image.texture_format).unwrap();
        bind_groups.bind_group.insert(
            render_target.clone(),
            render_device.create_bind_group(
                "texture conversion bind group",
                bg_layout,
                &BindGroupEntries::sequential((&org_image.texture_view, &org_image.sampler)),
            ),
        );
    }

    commands.insert_resource(bind_groups);
}

fn render(
    pipeline: ResMut<ConvertPipeline>,
    render_targets: Res<RenderTargetImages>,
    pipeline_cache: ResMut<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    bind_groups: Res<ConvertBindGroups>,
) {
    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("convert images"),
    });

    for (image_handle, bind_group) in bind_groups.bind_group.iter() {
        let Some(gpu_image) = gpu_images.get(image_handle) else {
            continue;
        };

        let Some(conv_handle) = render_targets.images.get(image_handle) else {
            continue;
        };

        let Some(conv_image) = gpu_images.get(conv_handle) else {
            continue;
        };

        let Some((pipeline, _)) = pipeline.pipeline.get(&gpu_image.texture_format) else {
            continue;
        };

        let Some(pipeline) = pipeline_cache.get_render_pipeline(*pipeline) else {
            continue;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Convert image pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &conv_image.texture_view,
                resolve_target: None,
                ops: Operations {
                    load: bevy::render::render_resource::LoadOp::Clear(Color::RED),
                    store: bevy::render::render_resource::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    let command = encoder.finish();
    render_queue.submit(vec![command]);
}

fn override_images(
    renderer: Res<VelloRenderer>,
    render_target_images: Res<RenderTargetImages>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let Ok(mut renderer) = renderer.try_lock() else {
        return;
    };
    for (org_image, conv_image) in render_target_images.images.iter() {
        let Some(gpu_image) = gpu_images.get(conv_image) else {
            continue;
        };

        let Some(image) = render_target_images.vello_images.get(org_image) else {
            continue;
        };

        renderer.override_image(
            image,
            Some(TexelCopyTextureInfoBase {
                texture: (*gpu_image.texture).clone(),
                mip_level: 0,
                origin: Origin3d { x: 0, y: 0, z: 0 },
                aspect: bevy_vello::vello::wgpu::TextureAspect::All,
            }),
        );
    }
}
