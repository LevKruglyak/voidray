use crate::preamble::*;

#[derive(Default, Clone)]
pub struct Settings {
    render: RenderSettings,
}

#[derive(Clone, PartialEq)]
pub enum RenderMode {
    Normal,
    Full,
}

#[derive(Clone)]
pub struct RenderSettings {
    pub render_mode: RenderMode,
    pub firefly_clamp: Float,
    pub max_bounces: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::Full,
            firefly_clamp: 3.0,
            max_bounces: 10,
        }
    }
}
