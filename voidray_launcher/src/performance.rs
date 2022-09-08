use systemstat::{Platform, System};

pub struct EnginePerformance {
    pub system: System,
}

impl Default for EnginePerformance {
    fn default() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl EnginePerformance {
    pub fn begin_frame(&mut self) {}

    pub fn end_frame(&mut self) {}
}
