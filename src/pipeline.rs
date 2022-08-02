use bevy::{
    prelude::World,
    render::{
        render_resource::PipelineCache,
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

pub mod copy_pipeline;
pub mod diffuse_pipeline;
pub mod draw_sensor_pipeline;
pub mod slime_sim_pipeline;

pub trait Pipeline<'a> {
    type CreationSettings;
    type BindGroupSettings;
    type UpdateSettings;
    type ExecuteSettings;

    fn new(world: &mut World) -> Self;

    fn queue_bind_group(
        &mut self,
        render_device: &RenderDevice,
        settings: &Self::BindGroupSettings,
    );

    fn update(&self, queue: &RenderQueue, update: &Self::UpdateSettings);

    fn execute(
        &self,
        render_context: &mut RenderContext,
        pipeline_cache: &PipelineCache,
        state: &ComputeSlimeState,
        execute_settings: &Self::ExecuteSettings,
    );
}

use crate::compute_render_node::ComputeSlimeState;

pub use self::{
    copy_pipeline::CopyPipeline, diffuse_pipeline::DiffusePipeline,
    draw_sensor_pipeline::DrawSensorPipeline, slime_sim_pipeline::SlimeSimPipeline,
};
