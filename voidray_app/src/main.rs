use voidray_launcher::*;

struct VoidrayEngine {}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        Self {}
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1200, 1000),
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<VoidrayEngine>::run(options);
}
