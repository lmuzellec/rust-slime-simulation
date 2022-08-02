use std::borrow::Cow;

use bevy::{
    prelude::{AssetServer, Handle, Shader, World},
    render::{
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use crate::{
    compute_render_node::ComputeSlimeState,
    pipeline::Pipeline,
    types::{Agent, SizeSettings, SlimeSettings, TimeBuffer},
};

const PARTICLES_PER_GROUP: usize = 64;

pub struct SlimeSimPipeline {
    pub bind_group: Option<BindGroup>,
    pub bind_group_layout: BindGroupLayout,
    pub slime_sim_pipeline_id: CachedComputePipelineId,
    pub workgroup_size: u32,
}

pub struct SlimeSimSetup {
    pub num_agents: u32,
}

#[derive(Debug)]
pub struct SlimeSimBuffers<'a> {
    pub texture_view_read: &'a TextureView,
    pub texture_view_write: &'a TextureView,
    pub agents_buffer: Buffer,
    pub settings_buffer: Buffer,
    pub time_buffer: Buffer,
    pub size_buffers: Buffer,
}

impl<'a> Pipeline<'a> for SlimeSimPipeline {
    type CreationSettings = SlimeSimSetup;
    type BindGroupSettings = SlimeSimBuffers<'a>;
    type UpdateSettings = ();
    type ExecuteSettings = ();

    fn new(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let settings = world.resource::<Self::CreationSettings>();

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
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<SlimeSettings>() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<TimeBuffer>() as u64
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                (std::mem::size_of::<Agent>() * settings.num_agents as usize)
                                    as u64,
                            ),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
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
        let shader: Handle<Shader> = asset_server.load("slime_simulation.wgsl");

        let workgroup_size =
            ((settings.num_agents as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let slime_sim_pipeline_id =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: None,
                layout: Some(vec![bind_group_layout.clone()]),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("slime_main"),
            });

        Self {
            bind_group: None,
            bind_group_layout,
            slime_sim_pipeline_id,
            workgroup_size,
        }
    }

    fn queue_bind_group(&mut self, render_device: &RenderDevice, buffers: &SlimeSimBuffers) {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffers.size_buffers.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.settings_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffers.time_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffers.agents_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&buffers.texture_view_read),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(&buffers.texture_view_write),
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
                    .push_debug_group("Execute slime simulation pipeline");
                {
                    let mut pass = render_context
                        .command_encoder
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_bind_group(0, &bind_group, &[]);

                    let slime_sim_pipeline = pipeline_cache
                        .get_compute_pipeline(self.slime_sim_pipeline_id)
                        .unwrap();

                    pass.set_pipeline(slime_sim_pipeline);
                    pass.dispatch_workgroups(self.workgroup_size, 1, 1);
                }
                render_context.command_encoder.pop_debug_group();
            }
            None => panic!("Bind group not set"),
        }
    }
}
