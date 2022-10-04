use bevy::{
    prelude::World,
    render::{
        render_graph::*,
        render_resource::PipelineCache,
        renderer::{RenderContext, RenderQueue},
    },
};

use crate::{
    compute_plugin::ComputeSlimeTime,
    compute_slime_pipeline::{ComputeSlimePipeline, ComputeTimeUpdate},
    pipeline::Pipeline,
    types::{AppSettings, TimeBuffer},
    AppSettingsUpdated,
};

pub struct ComputeRenderNode {
    state: ComputeSlimeState,
}

impl Default for ComputeRenderNode {
    fn default() -> Self {
        ComputeRenderNode {
            state: ComputeSlimeState::Init,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ComputeSlimeState {
    Init,
    Loaded,
}

impl Node for ComputeRenderNode {
    fn update(&mut self, world: &mut World) {
        let compute_slime_pipeline = world.resource::<ComputeSlimePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_queue = world.resource::<RenderQueue>();
        let compute_slime_time = world.resource::<ComputeSlimeTime>();

        let app_settings = world.resource::<AppSettings>();
        let app_settings_updated = world.resource::<AppSettingsUpdated>();

        let time_buffer = TimeBuffer {
            time: compute_slime_time.0.seconds_since_startup() as f32,
            delta_time: compute_slime_time.0.delta_seconds(),
        };

        if app_settings_updated.0 {
            compute_slime_pipeline.update_settings(render_queue, app_settings);
        }

        compute_slime_pipeline.update_time(render_queue, &ComputeTimeUpdate { time_buffer });
        compute_slime_pipeline.update_state(pipeline_cache, &mut self.state);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let compute_slime_pipeline = world.resource::<ComputeSlimePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let app_settings = world.resource::<AppSettings>();

        compute_slime_pipeline.execute(render_context, pipeline_cache, &self.state, app_settings);

        Ok(())
    }
}
