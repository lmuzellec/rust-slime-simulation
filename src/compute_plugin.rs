use bevy::{
    prelude::*,
    prelude::{Commands, Plugin},
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::RenderAssets,
        render_graph::RenderGraph,
        renderer::RenderDevice,
        RenderApp, RenderStage,
    },
};

use crate::{
    compute_render_node::ComputeRenderNode,
    compute_slime_pipeline::{ComputeSlimeBindGroup, ComputeSlimePipeline},
    pipeline::Pipeline,
    types::AppSettings,
    AppSettingsUpdated, AppShouldReset,
};

pub struct ComputePlugin;

impl Plugin for ComputePlugin {
    fn build(&self, app: &mut App) {
        let app_settings = app.world.get_resource::<AppSettings>().cloned().unwrap();
        app.add_plugin(ExtractResourcePlugin::<AppSettings>::default());
        app.add_plugin(ExtractResourcePlugin::<AppShouldReset>::default());
        app.add_plugin(ExtractResourcePlugin::<AppSettingsUpdated>::default());
        app.add_plugin(ExtractResourcePlugin::<ComputeSlimeDisplayImage>::default());
        app.add_plugin(ExtractResourcePlugin::<ComputeSlimeTime>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(app_settings)
            .init_resource::<ComputeSlimePipeline>()
            .add_system_to_stage(RenderStage::Prepare, reload_pipeline)
            .add_system_to_stage(RenderStage::Queue, ComputePlugin::queue_bind_group);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("slime_simulation", ComputeRenderNode::default());
        render_graph
            .add_node_edge("slime_simulation", CAMERA_DRIVER)
            .unwrap();
    }
}

fn reload_pipeline(mut commands: Commands, app_should_reset: Res<AppShouldReset>) {
    if app_should_reset.0 {
        commands.remove_resource::<ComputeSlimePipeline>();
        commands.init_resource::<ComputeSlimePipeline>();
    }
}

impl ComputePlugin {
    fn queue_bind_group(
        mut _commands: Commands,
        mut compute_slime_pipeline: ResMut<ComputeSlimePipeline>,
        gpu_images: Res<RenderAssets<Image>>,
        render_device: Res<RenderDevice>,
        compute_slime_display_image: Res<ComputeSlimeDisplayImage>,
    ) {
        let display_texture_view = &gpu_images[&compute_slime_display_image.0].texture_view;

        let compute_settings = &ComputeSlimeBindGroup {
            display_texture_view,
        };

        compute_slime_pipeline.queue_bind_group(&render_device, compute_settings)
    }
}

#[derive(Clone, Deref, ExtractResource, Debug)]
pub struct ComputeSlimeDisplayImage(pub Handle<Image>);

#[derive(Deref)]
pub struct ComputeSlimeTime(pub Time);

impl ExtractResource for ComputeSlimeTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        ComputeSlimeTime(time.clone())
    }
}
