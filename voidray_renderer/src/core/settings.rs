use crate::preamble::*;

#[derive(Default, Clone)]
pub struct Settings {
    pub render: RenderSettings,
    pub color_management: ColorManagementSettings,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Tonemap {
    None,
    ACES,
    Reinhard,
    Filmic,
    Uncharted2,
}

impl Tonemap {
    pub fn as_i32(&self) -> i32 {
        match *self {
            Tonemap::None => 0,
            Tonemap::ACES => 1,
            Tonemap::Reinhard => 2,
            Tonemap::Filmic => 3,
            Tonemap::Uncharted2 => 4,
        }
    }
}

#[derive(Clone)]
pub struct ColorManagementSettings {
    pub tonemap: Tonemap,
    pub gamma: f32,
    pub exposure: f32,
    pub transparent: bool,
}

impl Default for ColorManagementSettings {
    fn default() -> Self {
        Self {
            tonemap: Tonemap::None,
            gamma: 2.2,
            exposure: 0.0,
            transparent: true,
        }
    }
}
