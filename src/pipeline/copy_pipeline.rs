use std::borrow::Cow;

use bevy::{
    prelude::{AssetServer, Handle, World},
    render::{
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use crate::{compute_render_node::ComputeSlimeState, pipeline::Pipeline, types::SizeSettings};

const COPY_SIZE: f32 = 8.0;

pub struct CopyPipeline {
    pub bind_group: Option<BindGroup>,
    pub bind_group_layout: BindGroupLayout,
    pub copy_pipeline_id: CachedComputePipelineId,
    pub workgroup_size: (u32, u32),
}

pub struct CopyTextureView<'a> {
    pub size_buffer: Buffer,
    pub texture_view_read: &'a TextureView,
    pub texture_view_write: &'a TextureView,
}

impl<'a> Pipeline<'a> for CopyPipeline {
    type CreationSettings = SizeSettings;
    type BindGroupSettings = CopyTextureView<'a>;
    type UpdateSettings = ();
    type ExecuteSettings = ();

    fn new(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let settings = world.resource::<SizeSettings>();

        let bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<SizeSettings>() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let asset_server = world.resource::<AssetServer>();
        let shader: Handle<Shader> = asset_server.load("copy.wgsl");

        let workgroup_size = (
            (settings.width as f32 / COPY_SIZE).ceil() as u32,
            (settings.height as f32 / COPY_SIZE).ceil() as u32,
        );

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let copy_pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![bind_group_layout.clone()]),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("copy_main"),
        });

        CopyPipeline {
            bind_group: None,
            bind_group_layout,
            copy_pipeline_id,
            workgroup_size,
        }
    }

    fn queue_bind_group(&mut self, render_device: &RenderDevice, settings: &CopyTextureView) {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: settings.size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&settings.texture_view_read),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&settings.texture_view_write),
                },
            ],
        });

        self.bind_group = Some(bind_group);
    }

    fn update(&self, _queue: &RenderQueue, _update: &Self::UpdateSettings) {}

    fn execute(
        &self,
        render_context: &mut RenderContext,
        pipeline_cache: &PipelineCache,
        _state: &ComputeSlimeState,
        _execute_settings: &Self::ExecuteSettings,
    ) {
        match &self.bind_group {
            Some(bind_group) => {
                render_context
                    .command_encoder
                    .push_debug_group("Execute copy pipeline");
                {
                    let mut pass = render_context
                        .command_encoder
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_bind_group(0, &bind_group, &[]);

                    let copy_pipeline = pipeline_cache
                        .get_compute_pipeline(self.copy_pipeline_id)
                        .unwrap();

                    pass.set_pipeline(copy_pipeline);
                    pass.dispatch_workgroups(self.workgroup_size.0, self.workgroup_size.1, 1);
                }
                render_context.command_encoder.pop_debug_group();
            }
            None => panic!("Bind group not set"),
        }
    }
}
