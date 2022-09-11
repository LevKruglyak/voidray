#![allow(dead_code, unused_variables)]

use std::thread;

use gui::{Editable, render_actions, engine_ui};
use voidray_common::{simple::Materials, Environments, Surfaces};
use voidray_launcher::gui_implementation::*;
use voidray_launcher::*;
use voidray_renderer::camera::Camera;
use voidray_renderer::color::*;
use voidray_renderer::rayon::prelude::*;
use voidray_renderer::render::iterative::iterative_render;
use voidray_renderer::render::renderer::Renderer;
use voidray_renderer::render::target::CpuRenderTarget;
use voidray_renderer::render::viewport::Viewport;
use voidray_renderer::scene::{Accelerable, Scene};
use voidray_renderer::settings::{Settings, RenderSettings};
use voidray_renderer::vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use voidray_renderer::vulkano::pipeline::graphics;
use voidray_renderer::vulkano::render_pass::Subpass;
use voidray_renderer::{preamble::*, rand};

mod gui;
mod widgets;

pub struct VoidrayEngine {
    pub target: Arc<CpuRenderTarget>,
    pub scene: Arc<RwLock<Scene>>,
    pub settings: Arc<RwLock<Settings>>,
    pub renderer: Arc<RwLock<Renderer>>,
    viewport: Viewport,
}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let api = context.api();
        let target = CpuRenderTarget::new(api.compute_queue(), [500, 500]);
        let mut scene = Scene::empty();

        let red = scene.add_material(Materials::metal(hex_color(0xE78999), 0.1));
        let yellow = scene.add_material(Materials::dielectric(1.5));
        let green = scene.add_material(Materials::metal(hex_color(0xB3E7AA), 0.1));
        let blue = scene.add_material(Materials::metal(hex_color(0x7CA3E7), 0.01));
        let grey = scene.add_material(Materials::metal(hex_color(0xAAAAAA), 0.01));
        let light_mtl = scene.add_material(Materials::colored_emissive(hex_color(0xFFFFFF), 8.0));

        let spheres = vec![
            (vec3!(0.5, 1.0, 4.0), red),
            (vec3!(3.15, 1.5, -0.7), yellow),
            (vec3!(0.1, 0.6, -2.0), green),
            (vec3!(-1.7, 1.1, -0.2), blue),
            (vec3!(1.2, 0.5, 0.4), grey),
        ];

        for (pos, mtl) in spheres {
            let sph = scene.add_analytic_surface(Surfaces::sphere(pos, pos.y));
            scene.add_object(mtl, sph);
        }

        let light = scene.add_analytic_surface(Surfaces::sphere(vec3!(1.2, 8.0, -1.5), 2.0));
        scene.add_object(light_mtl, light);

        scene.camera = Camera::look_at(
            vec3!(0.7166, 2.8803, -9.2992),
            vec3!(0.8673, 0.9557, 0.2095),
            vec3!(0.0, 1.0, 0.0),
            0.6911,
        );
        scene.camera.dof = Some((0.15, vec3!(0.1, 0.6, -2.0)));

        let gnd = scene.add_analytic_surface(Surfaces::ground_plane(0.0));
        let gnd_mat = scene.add_material(Materials::lambertian(GRAY(0.2)));
        scene.add_object(gnd_mat, gnd);

        scene.environment = Environments::hdri("assets/studio.exr");

        let settings = Settings::default();

        let scene = Arc::new(RwLock::new(scene));
        let settings = Arc::new(RwLock::new(settings));

        Self {
            target: target.clone(),
            scene: scene.clone(),
            settings: settings.clone(),
            viewport: Viewport::new(api.graphics_queue(), context.viewport_subpass(), target.clone()),
            renderer: Arc::new(RwLock::new(Renderer::new(api.compute_queue(), scene, settings, target))),
        }
    }

    fn immediate(
        &mut self,
        context: &mut <<Self as Engine>::Gui as GuiImplementation>::Context,
        api: &mut EngineApi,
    ) {
        engine_ui(self, context, api);
    }

    fn render(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        subpass: Subpass,
        viewport: graphics::viewport::Viewport,
        api: &mut EngineApi,
    ) {
        self.viewport.draw(command_buffer, viewport);
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
