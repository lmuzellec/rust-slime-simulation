use bevy::{
    prelude::{
        default, App, Assets, Camera2dBundle, ClearColor, Color, Commands, Image, Res, ResMut, Vec2,
    },
    render::{extract_resource::ExtractResource, render_resource::*},
    sprite::{Sprite, SpriteBundle},
    window::WindowDescriptor,
    DefaultPlugins,
};
use compute_plugin::{ComputePlugin, ComputeSlimeDisplayImage};
use gui_plugin::GuiPlugin;
use types::AppSettings;

mod compute_plugin;
mod compute_render_node;
mod compute_slime_pipeline;
mod gui_plugin;
mod pipeline;
mod types;

const SIZE: (u32, u32) = (1280, 720);

#[derive(Clone, Copy, ExtractResource)]
struct AppShouldReset(bool);

#[derive(Clone, Copy, ExtractResource)]
struct AppSettingsUpdated(bool);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: SIZE.0 as f32,
            height: SIZE.1 as f32,
            ..default()
        })
        .insert_resource(AppSettings::default())
        .insert_resource(AppShouldReset(false))
        .insert_resource(AppSettingsUpdated(false))
        .add_plugins(DefaultPlugins)
        .add_plugin(GuiPlugin)
        .add_plugin(ComputePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    app_settings: Res<AppSettings>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: app_settings.width,
            height: app_settings.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0, 0, 0, 0, 0],
        TextureFormat::Rgba16Float,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image_handle = images.add(image);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                app_settings.width as f32,
                app_settings.height as f32,
            )),
            ..default()
        },
        texture: image_handle.clone(),
        ..default()
    });

    commands.insert_resource(ComputeSlimeDisplayImage(image_handle));
    commands.spawn_bundle(Camera2dBundle::default());
}
