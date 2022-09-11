use crate::preamble::*;

#[derive(Default, Clone)]
pub struct Settings {
    pub render: RenderSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderMode {
    Normal,
    Full,
}

#[derive(Clone)]
pub struct RenderSettings {
    pub total_samples: u32,
    pub update_frequency: f32,
    pub render_mode: RenderMode,
    pub firefly_clamp: Float,
    pub max_bounces: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            total_samples: 100,
            update_frequency: 0.5,
            render_mode: RenderMode::Full,
            firefly_clamp: 3.0,
            max_bounces: 10,
        }
    }
}
