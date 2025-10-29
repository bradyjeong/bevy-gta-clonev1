use crate::components::UnderwaterSettings;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin,
};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    binding_types::{sampler, texture_2d, texture_depth_2d, uniform_buffer},
    *,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::view::{
    ViewDepthTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms,
};
use bevy::render::{Render, RenderApp, RenderSet};

pub struct UnderwaterPlugin;

impl Plugin for UnderwaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<UnderwaterSettings>::default())
            .add_plugins(UniformComponentPlugin::<UnderwaterSettings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Render,
                configure_underwater_depth_texture.in_set(RenderSet::ManageViews),
            )
            .add_render_graph_node::<ViewNodeRunner<UnderwaterNode>>(Core3d, UnderwaterLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::MainTransparentPass,
                    UnderwaterLabel,
                    Node3d::Tonemapping,
                ),
            );

        #[cfg(feature = "debug-ui")]
        info!("✅ Underwater post-processing plugin initialized");
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<UnderwaterPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct UnderwaterLabel;

fn configure_underwater_depth_texture(mut cameras: Query<&mut Camera3d, With<UnderwaterSettings>>) {
    for mut camera_3d in cameras.iter_mut() {
        let mut depth_texture_usages = TextureUsages::from(camera_3d.depth_texture_usages);
        depth_texture_usages |= TextureUsages::TEXTURE_BINDING;
        camera_3d.depth_texture_usages = depth_texture_usages.into();
    }
}

#[derive(Default)]
struct UnderwaterNode;

impl ViewNode for UnderwaterNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static DynamicUniformIndex<UnderwaterSettings>,
        &'static ViewUniformOffset,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, depth, settings_index, view_uniform_offset): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_resource = world.resource::<UnderwaterPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let settings_uniforms = world.resource::<ComponentUniforms<UnderwaterSettings>>();
        let view_uniforms = world.resource::<ViewUniforms>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_resource.pipeline_id)
        else {
            warn_once!("Underwater pipeline not ready yet");
            return Ok(());
        };

        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            warn_once!("Settings uniforms not ready");
            return Ok(());
        };

        let Some(view_binding) = view_uniforms.uniforms.binding() else {
            warn_once!("View uniforms not ready");
            return Ok(());
        };

        debug_once!("Underwater node running successfully");

        let post_process = view_target.post_process_write();

        let view_bind_group = render_context.render_device().create_bind_group(
            "underwater_view_bind_group",
            &pipeline_resource.view_layout,
            &BindGroupEntries::sequential((view_binding.clone(),)),
        );

        let src_bind_group = render_context.render_device().create_bind_group(
            "underwater_src_bind_group",
            &pipeline_resource.src_layout,
            &BindGroupEntries::sequential((post_process.source, &pipeline_resource.sampler)),
        );

        let depth_bind_group = render_context.render_device().create_bind_group(
            "underwater_depth_bind_group",
            &pipeline_resource.depth_layout,
            &BindGroupEntries::sequential((depth.view(),)),
        );

        let settings_bind_group = render_context.render_device().create_bind_group(
            "underwater_settings_bind_group",
            &pipeline_resource.settings_layout,
            &BindGroupEntries::sequential((settings_binding.clone(),)),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("underwater_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &view_bind_group, &[view_uniform_offset.offset]);
        render_pass.set_bind_group(1, &src_bind_group, &[]);
        render_pass.set_bind_group(2, &depth_bind_group, &[]);
        render_pass.set_bind_group(3, &settings_bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct UnderwaterPipeline {
    view_layout: BindGroupLayout,
    src_layout: BindGroupLayout,
    depth_layout: BindGroupLayout,
    settings_layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for UnderwaterPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(
            "underwater_view_layout",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
        );

        let src_layout = render_device.create_bind_group_layout(
            "underwater_src_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let depth_layout = render_device.create_bind_group_layout(
            "underwater_depth_layout",
            &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (texture_depth_2d(),)),
        );

        let settings_layout = render_device.create_bind_group_layout(
            "underwater_settings_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (uniform_buffer::<UnderwaterSettings>(true),),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("underwater_src_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            ..default()
        });

        let shader = world.load_asset("shaders/underwater_post.wgsl");

        let pipeline_id =
            world
                .resource::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("underwater_pipeline".into()),
                    layout: vec![
                        view_layout.clone(),
                        src_layout.clone(),
                        depth_layout.clone(),
                        settings_layout.clone(),
                    ],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::Rgba8UnormSrgb,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            view_layout,
            src_layout,
            depth_layout,
            settings_layout,
            sampler,
            pipeline_id,
        }
    }
}
