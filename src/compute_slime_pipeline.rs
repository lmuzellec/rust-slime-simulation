use bevy::{
    prelude::{FromWorld, World},
    render::{
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};
use copy_pipeline::CopyPipeline;
use diffuse_pipeline::DiffusePipeline;
use draw_sensor_pipeline::DrawSensorPipeline;
use rand::Rng;

use crate::{
    compute_render_node::ComputeSlimeState,
    pipeline::{
        copy_pipeline::{self, CopyTextureView},
        diffuse_pipeline::{self, DiffuseBuffers},
        draw_sensor_pipeline::{self, DrawSensorBuffers},
        slime_sim_pipeline::{SlimeSimBuffers, SlimeSimSetup},
        Pipeline, SlimeSimPipeline,
    },
    types::{
        Agent, AgentDistribution, AppSettings, DiffuseSettings, SizeSettings, SlimeSettings,
        TimeBuffer,
    },
};

pub struct ComputeSlimePipeline {
    pub state: ComputeSlimeState,

    pub copy_render_display_to_display_pipeline: CopyPipeline,
    pub copy_display_to_trail_pipeline: CopyPipeline,
    pub slime_sim_pipeline: SlimeSimPipeline,
    pub diffuse_pipeline: DiffusePipeline,
    pub copy_diffuse_to_display_pipeline: CopyPipeline,
    pub copy_display_to_render_display_pipeline: CopyPipeline,

    pub copy_display_to_sensor_pipeline: CopyPipeline,
    pub draw_sensor_pipeline: DrawSensorPipeline,
    pub copy_sensor_to_render_display_pipeline: CopyPipeline,

    pub agents_buffer: Buffer,
    pub size_buffer: Buffer,
    pub diffuse_buffer: Buffer,
    pub settings_buffer: Buffer,
    pub time_buffer: Buffer,

    pub display_texture_view: TextureView,
    pub trail_texture_view: TextureView,
    pub diffuse_texture_view: TextureView,
    pub sensor_texture_view: TextureView,
}

pub struct ComputeSlimeBindGroup<'a> {
    pub display_texture_view: &'a TextureView,
}

pub struct ComputeTimeUpdate {
    pub time_buffer: TimeBuffer,
}

impl ComputeSlimePipeline {
    pub fn update_settings(&self, queue: &RenderQueue, app_settings: &AppSettings) {
        let diffuse_settings = DiffuseSettings {
            decay_rate: app_settings.decay_rate,
            diffuse_rate: app_settings.diffuse_rate,
        };

        queue.write_buffer(
            &self.diffuse_buffer,
            0,
            bytemuck::bytes_of(&diffuse_settings),
        );

        let slime_settings = SlimeSettings {
            num_agents: app_settings.num_agents,
            trail_weight: app_settings.trail_weight,
            memory_offset_1: 0,
            memory_offset_2: 0,
            species_settings: app_settings.species_settings,
        };

        queue.write_buffer(
            &self.settings_buffer,
            0,
            bytemuck::bytes_of(&slime_settings),
        );
    }

    pub fn update_time(&self, queue: &RenderQueue, update: &ComputeTimeUpdate) {
        queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::bytes_of(&update.time_buffer),
        );
    }

    pub fn update_state(&self, pipeline_cache: &PipelineCache, state: &mut ComputeSlimeState) {
        match state {
            ComputeSlimeState::Init => {
                if let (
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                    CachedPipelineState::Ok(_),
                ) = (
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_render_display_to_display_pipeline
                            .copy_pipeline_id,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_display_to_trail_pipeline.copy_pipeline_id,
                    ),
                    pipeline_cache
                        .get_compute_pipeline_state(self.slime_sim_pipeline.slime_sim_pipeline_id),
                    pipeline_cache
                        .get_compute_pipeline_state(self.diffuse_pipeline.diffuse_pipeline_id),
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_diffuse_to_display_pipeline.copy_pipeline_id,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_display_to_render_display_pipeline
                            .copy_pipeline_id,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_display_to_sensor_pipeline.copy_pipeline_id,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        self.draw_sensor_pipeline.draw_sensor_pipeline_id,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        self.copy_sensor_to_render_display_pipeline.copy_pipeline_id,
                    ),
                ) {
                    *state = ComputeSlimeState::Loaded;
                }
            }
            ComputeSlimeState::Loaded => {}
        }
    }
}

impl<'a> Pipeline<'a> for ComputeSlimePipeline {
    type CreationSettings = AppSettings;
    type BindGroupSettings = ComputeSlimeBindGroup<'a>;
    type ExecuteSettings = AppSettings;

    fn new(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let app_settings = world.resource::<AppSettings>();

        let agents: Vec<Agent> = match app_settings.agent_distribution {
            AgentDistribution::InnerCircle => (0..app_settings.num_agents)
                .into_iter()
                .map(|i| {
                    let angle: f32 = 2.0 * 3.1415 / (app_settings.num_agents as f32) * i as f32;
                    let x = app_settings.width as f32 / 2.0 + angle.cos() * 200.0;
                    let y = app_settings.height as f32 / 2.0 + angle.sin() * 200.0;
                    Agent {
                        position: [x, y],
                        angle: 3.1415 + angle,
                        species_index: 0,
                    }
                })
                .collect::<Vec<_>>(),
            AgentDistribution::OuterCircle => (0..app_settings.num_agents)
                .into_iter()
                .map(|i| {
                    let angle: f32 = 2.0 * 3.1415 / (app_settings.num_agents as f32) * i as f32;
                    let x = app_settings.width as f32 / 2.0 + angle.cos() * 200.0;
                    let y = app_settings.height as f32 / 2.0 + angle.sin() * 200.0;
                    Agent {
                        position: [x, y],
                        angle: angle,
                        species_index: 0,
                    }
                })
                .collect::<Vec<_>>(),
            AgentDistribution::InnerDisk => (0..app_settings.num_agents)
                .into_iter()
                .map(|_| {
                    let angle: f32 = rand::thread_rng().gen::<f32>() * 2.0 * 3.1415;
                    let x = app_settings.width as f32 / 2.0
                        + angle.cos() * rand::thread_rng().gen::<f32>() * 200.0;
                    let y = app_settings.height as f32 / 2.0
                        + angle.sin() * rand::thread_rng().gen::<f32>() * 200.0;
                    Agent {
                        position: [x, y],
                        angle: angle + 3.1415,
                        species_index: 0,
                    }
                })
                .collect::<Vec<_>>(),
            AgentDistribution::Random => (0..app_settings.num_agents)
                .into_iter()
                .map(|_| {
                    let angle: f32 = rand::thread_rng().gen::<f32>() * 2.0 * 3.1415;
                    let x = rand::thread_rng().gen::<f32>() * app_settings.width as f32;
                    let y = rand::thread_rng().gen::<f32>() * app_settings.height as f32;
                    Agent {
                        position: [x, y],
                        angle: angle,
                        species_index: 0,
                    }
                })
                .collect::<Vec<_>>(),
        };

        let size_settings = SizeSettings {
            width: app_settings.width,
            height: app_settings.height,
        };

        let diffuse_settings = DiffuseSettings {
            decay_rate: app_settings.decay_rate,
            diffuse_rate: app_settings.diffuse_rate,
        };

        let slime_settings = SlimeSettings {
            num_agents: app_settings.num_agents,
            trail_weight: app_settings.trail_weight,
            memory_offset_1: 0,
            memory_offset_2: 0,

            species_settings: app_settings.species_settings,
        };

        let time = TimeBuffer {
            time: 0.0,
            delta_time: 0.0,
        };

        let agents_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Agents buffer"),
            contents: bytemuck::cast_slice(&agents),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let size_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Size buffer"),
            contents: bytemuck::bytes_of(&size_settings),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let diffuse_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Diffuse buffer"),
            contents: bytemuck::bytes_of(&diffuse_settings),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let settings_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Settings buffer"),
            contents: bytemuck::bytes_of(&slime_settings),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let time_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Time buffer"),
            contents: bytemuck::bytes_of(&time),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let display_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: app_settings.width,
                height: app_settings.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
        });

        let display_texture_view = display_texture.create_view(&TextureViewDescriptor::default());

        let trail_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: app_settings.width,
                height: app_settings.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
        });

        let trail_texture_view = trail_texture.create_view(&TextureViewDescriptor::default());

        let diffuse_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: app_settings.width,
                height: app_settings.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
        });

        let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());

        let sensor_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: app_settings.width,
                height: app_settings.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
        });

        let sensor_texture_view = sensor_texture.create_view(&TextureViewDescriptor::default());

        let slime_sim_setup = SlimeSimSetup {
            num_agents: app_settings.num_agents,
        };

        world.insert_resource(slime_sim_setup);
        world.insert_resource(size_settings);

        let copy_render_display_to_display_pipeline = CopyPipeline::new(world);
        let copy_display_to_trail_pipeline = CopyPipeline::new(world);
        let slime_sim_pipeline = SlimeSimPipeline::new(world);
        let diffuse_pipeline = DiffusePipeline::new(world);
        let copy_diffuse_to_display_pipeline = CopyPipeline::new(world);
        let copy_display_to_render_display_pipeline = CopyPipeline::new(world);

        let copy_display_to_sensor_pipeline = CopyPipeline::new(world);
        let draw_sensor_pipeline = DrawSensorPipeline::new(world);
        let copy_sensor_to_render_display_pipeline = CopyPipeline::new(world);

        Self {
            state: ComputeSlimeState::Init,

            copy_render_display_to_display_pipeline,
            copy_display_to_trail_pipeline,
            slime_sim_pipeline,
            diffuse_pipeline,
            copy_diffuse_to_display_pipeline,
            copy_display_to_render_display_pipeline,

            copy_display_to_sensor_pipeline,
            draw_sensor_pipeline,
            copy_sensor_to_render_display_pipeline,

            agents_buffer,
            size_buffer,
            diffuse_buffer,
            settings_buffer,
            time_buffer,

            display_texture_view,
            trail_texture_view,
            diffuse_texture_view,
            sensor_texture_view,
        }
    }

    fn queue_bind_group(&mut self, render_device: &RenderDevice, settings: &ComputeSlimeBindGroup) {
        let copy_render_display_to_display = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: settings.display_texture_view,
            texture_view_write: &self.display_texture_view,
        };

        self.copy_render_display_to_display_pipeline
            .queue_bind_group(render_device, &copy_render_display_to_display);

        let copy_display_to_trail = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: &self.display_texture_view,
            texture_view_write: &self.trail_texture_view,
        };

        self.copy_display_to_trail_pipeline
            .queue_bind_group(render_device, &copy_display_to_trail);

        let slime_sim_buffers = SlimeSimBuffers {
            size_buffers: self.size_buffer.clone(),
            agents_buffer: self.agents_buffer.clone(),
            settings_buffer: self.settings_buffer.clone(),
            time_buffer: self.time_buffer.clone(),
            texture_view_read: &self.display_texture_view,
            texture_view_write: &self.trail_texture_view,
        };

        self.slime_sim_pipeline
            .queue_bind_group(render_device, &slime_sim_buffers);

        let diffuse_buffers = DiffuseBuffers {
            size_buffer: self.size_buffer.clone(),
            diffuse_buffer: self.diffuse_buffer.clone(),
            time_buffer: self.time_buffer.clone(),
            trail_texture: &self.trail_texture_view,
            diffuse_texture: &self.diffuse_texture_view,
        };

        self.diffuse_pipeline
            .queue_bind_group(render_device, &diffuse_buffers);

        let copy_diffuse_to_display = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: &self.diffuse_texture_view,
            texture_view_write: &self.display_texture_view,
        };

        self.copy_diffuse_to_display_pipeline
            .queue_bind_group(render_device, &copy_diffuse_to_display);

        let copy_display_to_render_display = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: &self.display_texture_view,
            texture_view_write: settings.display_texture_view,
        };

        self.copy_display_to_render_display_pipeline
            .queue_bind_group(render_device, &copy_display_to_render_display);

        let copy_display_to_sensor = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: &self.display_texture_view,
            texture_view_write: &self.sensor_texture_view,
        };

        self.copy_display_to_sensor_pipeline
            .queue_bind_group(render_device, &copy_display_to_sensor);

        let draw_sensor = DrawSensorBuffers {
            size_buffers: self.size_buffer.clone(),
            settings_buffer: self.settings_buffer.clone(),
            agents_buffer: self.agents_buffer.clone(),
            texture_view_read: &self.display_texture_view,
            texture_view_write: &self.sensor_texture_view,
        };

        self.draw_sensor_pipeline
            .queue_bind_group(render_device, &draw_sensor);

        let copy_sensor_to_render_display = CopyTextureView {
            size_buffer: self.size_buffer.clone(),
            texture_view_read: &self.sensor_texture_view,
            texture_view_write: settings.display_texture_view,
        };

        self.copy_sensor_to_render_display_pipeline
            .queue_bind_group(render_device, &copy_sensor_to_render_display);
    }

    fn execute(
        &self,
        render_context: &mut RenderContext,
        pipeline_cache: &PipelineCache,
        state: &ComputeSlimeState,
        app_settings: &AppSettings,
    ) {
        match state {
            ComputeSlimeState::Init => {
                self.copy_render_display_to_display_pipeline.execute(
                    render_context,
                    pipeline_cache,
                    state,
                    &(),
                );
            }
            ComputeSlimeState::Loaded => {
                self.copy_display_to_trail_pipeline.execute(
                    render_context,
                    pipeline_cache,
                    state,
                    &(),
                );
                self.slime_sim_pipeline
                    .execute(render_context, pipeline_cache, state, &());
                self.diffuse_pipeline
                    .execute(render_context, pipeline_cache, state, &());
                self.copy_diffuse_to_display_pipeline.execute(
                    render_context,
                    pipeline_cache,
                    state,
                    &(),
                );

                if app_settings.render_sensors {
                    self.copy_display_to_sensor_pipeline.execute(
                        render_context,
                        pipeline_cache,
                        state,
                        &(),
                    );
                    self.draw_sensor_pipeline
                        .execute(render_context, pipeline_cache, state, &());
                    self.copy_sensor_to_render_display_pipeline.execute(
                        render_context,
                        pipeline_cache,
                        state,
                        &(),
                    );
                } else {
                    self.copy_display_to_render_display_pipeline.execute(
                        render_context,
                        pipeline_cache,
                        state,
                        &(),
                    );
                }
            }
        }
    }
}

impl FromWorld for ComputeSlimePipeline {
    fn from_world(world: &mut World) -> Self {
        ComputeSlimePipeline::new(world)
    }
}
