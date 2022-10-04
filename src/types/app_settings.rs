use bevy::render::extract_resource::ExtractResource;

use crate::SIZE;

use super::SpeciesSettings;

#[derive(Clone, Copy, ExtractResource, Debug)]
pub struct AppSettings {
    pub app_preset: AppPreset,
    pub agent_distribution: AgentDistribution,

    pub width: u32,
    pub height: u32,
    pub num_agents: u32,

    pub trail_weight: f32,
    pub decay_rate: f32,
    pub diffuse_rate: f32,

    pub render_sensors: bool,

    pub species_settings: [SpeciesSettings; 4],
}

impl AppSettings {
    pub fn apply_preset(&mut self) {
        match self.app_preset {
            AppPreset::Default => {
                self.agent_distribution = AgentDistribution::InnerCircle;

                self.reset_settings();
            }
            AppPreset::SuperNova => {
                self.agent_distribution = AgentDistribution::InnerCircle;

                self.reset_settings();
            }
        }
    }

    pub fn reset_settings(&mut self) {
        match self.app_preset {
            AppPreset::Default => {
                self.width = SIZE.0;
                self.height = SIZE.1;
                self.num_agents = 100000;

                self.trail_weight = 1.0;
                self.decay_rate = 0.75;
                self.diffuse_rate = 5.0;

                self.render_sensors = false;

                self.species_settings = [SpeciesSettings::default(); 4];
            }
            AppPreset::SuperNova => {
                self.width = SIZE.0;
                self.height = SIZE.1;
                self.num_agents = 100000;

                self.trail_weight = 1.0;
                self.decay_rate = 0.75;
                self.diffuse_rate = 5.0;

                self.render_sensors = false;

                self.species_settings = [SpeciesSettings {
                    turn_speed: 1.0,
                    ..Default::default()
                }; 4];
            }
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_preset: AppPreset::Default,
            agent_distribution: AgentDistribution::InnerCircle,

            width: SIZE.0,
            height: SIZE.1,
            num_agents: 100000,

            trail_weight: 1.0,
            decay_rate: 0.75,
            diffuse_rate: 5.0,

            render_sensors: false,

            species_settings: [SpeciesSettings::default(); 4],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppPreset {
    Default,
    SuperNova,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AgentDistribution {
    InnerCircle,
    OuterCircle,
    InnerDisk,
    Random,
}
