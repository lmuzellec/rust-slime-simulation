use bevy::prelude::{App, Input, KeyCode, Plugin, Res, ResMut};
use bevy_egui::{
    egui::{ComboBox, DragValue, Ui, Window},
    EguiContext, EguiPlugin,
};

use crate::{
    types::{AgentDistribution, AppPreset, AppSettings, SpeciesSettings},
    AppSettingsUpdated, AppShouldReset,
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .init_resource::<WindowState>()
            .add_system(update_window_open)
            .add_system(egui_system);
    }
}

#[derive(Default)]
struct WindowState {
    is_window_open: bool,
}

fn update_window_open(keyboard_input: Res<Input<KeyCode>>, mut window_state: ResMut<WindowState>) {
    if keyboard_input.just_pressed(KeyCode::E) {
        window_state.is_window_open = !window_state.is_window_open;
    }
}

fn egui_system(
    mut window_state: ResMut<WindowState>,
    mut egui_context: ResMut<EguiContext>,
    mut app_settings: ResMut<AppSettings>,
    mut app_should_reset: ResMut<AppShouldReset>,
    mut app_settings_updated: ResMut<AppSettingsUpdated>,
) {
    Window::new("Slime Simulation")
        .resizable(false)
        .open(&mut window_state.is_window_open)
        .show(egui_context.ctx_mut(), |ui| {
            let mut reset_simulation = false;
            let mut settings_updated = false;

            ComboBox::from_label("Simulation presets")
                .selected_text(format!("{:?}", app_settings.app_preset))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app_settings.app_preset,
                        AppPreset::Default,
                        "Default",
                    );
                    ui.selectable_value(
                        &mut app_settings.app_preset,
                        AppPreset::SuperNova,
                        "SuperNova",
                    );
                });

            if ui.button("Apply preset to Simulation and Reset").clicked() {
                app_settings.apply_preset();
                reset_simulation = true;
            }

            ui.separator();

            ComboBox::from_label("Agent distribution at creation")
                .selected_text(format!("{:?}", app_settings.agent_distribution))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app_settings.agent_distribution,
                        AgentDistribution::InnerCircle,
                        "Inner Circle",
                    );
                    ui.selectable_value(
                        &mut app_settings.agent_distribution,
                        AgentDistribution::OuterCircle,
                        "Outer Circle",
                    );
                    ui.selectable_value(
                        &mut app_settings.agent_distribution,
                        AgentDistribution::InnerDisk,
                        "Inner Disk",
                    );
                    ui.selectable_value(
                        &mut app_settings.agent_distribution,
                        AgentDistribution::Random,
                        "Random",
                    );
                });

            if ui.button("Reset Simulation").clicked() {
                reset_simulation = true;
            }

            ui.separator();

            if ui.button("Reload settings").clicked() {
                app_settings.reset_settings();
                settings_updated = true;
            };

            settings_updated = settings_updated
                || ui
                    .checkbox(&mut app_settings.render_sensors, "render agent sense")
                    .changed();

            ui.horizontal(|ui| {
                settings_updated = settings_updated
                    || ui
                        .add(DragValue::new(&mut app_settings.decay_rate).speed(0.01))
                        .changed();
                ui.label("Decay rate");
            });

            ui.horizontal(|ui| {
                settings_updated = settings_updated
                    || ui
                        .add(DragValue::new(&mut app_settings.diffuse_rate).speed(0.01))
                        .changed();
                ui.label("Diffuse rate");
            });

            ui.separator();

            ui.heading("Species settings");

            add_species_settings(
                &mut app_settings.species_settings[0],
                &mut settings_updated,
                ui,
                0,
            );
            add_species_settings(
                &mut app_settings.species_settings[1],
                &mut settings_updated,
                ui,
                1,
            );
            add_species_settings(
                &mut app_settings.species_settings[2],
                &mut settings_updated,
                ui,
                2,
            );
            add_species_settings(
                &mut app_settings.species_settings[3],
                &mut settings_updated,
                ui,
                3,
            );

            app_should_reset.0 = reset_simulation;
            app_settings_updated.0 = settings_updated;
        });
}

fn add_species_settings(
    species_settings: &mut SpeciesSettings,
    changed: &mut bool,
    ui: &mut Ui,
    species_index: usize,
) {
    ui.collapsing(format!("Species #{}", species_index + 1), |ui| {
        ui.horizontal(|ui| {
            *changed = *changed
                || ui
                    .add(DragValue::new(&mut species_settings.move_speed).speed(0.01))
                    .changed();
            ui.label("Move speed");
        });

        ui.horizontal(|ui| {
            *changed = *changed
                || ui
                    .add(DragValue::new(&mut species_settings.turn_speed).speed(0.01))
                    .changed();
            ui.label("Turn speed");
        });

        ui.horizontal(|ui| {
            *changed = *changed
                || ui
                    .add(DragValue::new(&mut species_settings.sensor_angle_spacing).speed(0.01))
                    .changed();
            ui.label("Sensor angle spacing");
        });

        ui.horizontal(|ui| {
            *changed = *changed
                || ui
                    .add(DragValue::new(&mut species_settings.sensor_offset_dst).speed(0.01))
                    .changed();
            ui.label("Sensor range");
        });

        ui.horizontal(|ui| {
            *changed = *changed
                || ui
                    .add(DragValue::new(&mut species_settings.sensor_size).speed(1))
                    .changed();
            ui.label("Sensor size");
        });
    });
}
