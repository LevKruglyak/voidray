#![allow(dead_code, unused_variables)]

use std::thread;
use std::time::Duration;

use voidray_common::{Environments, Surfaces, MicrofacetBSDF, DisneyBSDF};
use voidray_launcher::gui_implementation::*;
use voidray_launcher::*;
use voidray_renderer::camera::Camera;
use voidray_renderer::color::{GRAY, BLACK, hex_color};
use voidray_renderer::render::iterative::iterative_render;
use voidray_renderer::render::target::CpuRenderTarget;
use voidray_renderer::render::viewport::Viewport;
use voidray_renderer::scene::{Scene, Accelerable};
use voidray_renderer::settings::Settings;
use voidray_renderer::traits::Surface;
use voidray_renderer::vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use voidray_renderer::vulkano::pipeline::graphics;
use voidray_renderer::vulkano::render_pass::Subpass;
use voidray_renderer::{preamble::*, rand};
use voidray_renderer::rayon::prelude::*;

struct VoidrayEngine {
    target: Arc<RwLock<CpuRenderTarget>>,
    scene: Scene,
    viewport: Viewport,
}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let api = context.api();
        let target = CpuRenderTarget::new(api.compute_queue(), [300, 300]);
        let mut scene = Scene::empty();

        // let red = scene.add_material(MicrofacetBSDF::specular(hex_color(0xE78999), 0.1));
        // let yellow = scene.add_material(Arc::new(Dielectric::new(1.5)));
        // let green = scene.add_material(MicrofacetBSDF::specular(hex_color(0xB3E7AA), 0.1));
        // let blue = scene.add_material(MicrofacetBSDF::specular(hex_color(0x7CA3E7), 0.01));
        // let grey = scene.add_material(MicrofacetBSDF::specular(hex_color(0xAAAAAA), 0.01));
        // let light_mtl = scene.add_material(MicrofacetBSDF::light(hex_color(0xFFFFFF), 8.0));
        //
        // let spheres = vec![
        //     (vec3!(0.5, 4.0, 1.0), red),
        //     (vec3!(3.15, -0.7, 1.5), yellow),
        //     (vec3!(0.1, -2.0, 0.6), green),
        //     (vec3!(-1.7, -0.2, 1.1), blue),
        //     (vec3!(1.2, 0.4, 0.5), grey),
        // ];
        //
        // for (pos, mtl) in spheres {
        //     let sph = scene.add_analytic_surface(Surfaces::sphere(pos, pos.z));
        //     scene.add_object(mtl, sph);
        // }
        //
        // let light = scene.add_analytic_surface(Surfaces::sphere(vec3!(1.2, -1.5, 8.0), 2.0));
        // scene.add_object(light_mtl, light);
        //
        // scene.camera = Camera::look_at(
        //     vec3!(0.7166, -9.2992, 2.8803),
        //     vec3!(0.8673, 0.2095, 0.9557),
        //     vec3!(0.0, 0.0, 1.0),
        //     0.6911);
        scene.camera.dof = None;

        scene.environment = Environments::hdri("assets/studio.exr");
        // scene.environment = Environments::uniform(GRAY(0.1));
        
        let red = scene.add_material(MicrofacetBSDF::diffuse(hex_color(0xFF0A1C)));
        let sph = scene.add_analytic_surface(Surfaces::sphere(vec3!(0.0), 1.0));
        let sph = scene.add_object(red, sph);

        let gray = scene.add_material(MicrofacetBSDF::diffuse(hex_color(0xA0A0A0)));
        let sph = scene.add_analytic_surface(Surfaces::sphere(vec3!(0.0, -11.0, 0.0), 10.0));
        let sph = scene.add_object(gray, sph);

        Self {
            target: target.clone(),
            scene,
            viewport: Viewport::new(api.graphics_queue(), context.viewport_subpass(), target),
        }
    }

    fn immediate(
        &mut self,
        context: &mut <<Self as Engine>::Gui as GuiImplementation>::Context,
        api: &mut EngineApi,
    ) {
        egui::Window::new("Hello, World! ").show(context, |ui| {
            ui.label("This is a test of the window!");
            if ui.button("Do some bullshit").clicked() {
                let target = self.target.clone();
                let dimensions = { self.target.read().unwrap().dimensions() };

                let color: (f32, f32, f32) = (
                    0.5 * rand::random::<f32>() + 0.5,
                    0.5 * rand::random::<f32>() + 0.5,
                    0.5 * rand::random::<f32>() + 0.5,
                );
                let speed = (rand::random::<f32>() * 1000.0) as u64;

                thread::spawn(move || {
                    target.write().unwrap().buffer().as_slice_mut().par_iter_mut().for_each(|color| {
                        *color = rand::random();
                    });
                    target.write().unwrap().try_push();
                });
            }

            ui.label("This is a test of the window!");
            if ui.button("Do some other bullshit").clicked() {
                let target = self.target.clone();
                let scene = self.scene.build_acceleration();
                let settings = Settings::default().render;

                target.write().unwrap().clear();

                let total_samples = 1000;
                thread::spawn(move || {
                    for _ in 0..total_samples {
                        iterative_render(target.clone(), &scene, &settings, 1, total_samples);
                    }
                });
            }

            if ui.button("Clear").clicked() {
                self.target.write().unwrap().clear();
            }

            if ui.button("Random resize").clicked() {
                let width: u32 = rand::random::<u32>() % 1000;
                self.target.write().unwrap().resize([width, width]);
            }

            if ui.button("Clear").clicked() {
                self.target.write().unwrap().clear();
            }

            if ui.button("Random resize").clicked() {
                let width: u32 = rand::random::<u32>() % 1000;
                self.target.write().unwrap().resize([width, width]);
            }
        });
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
            dimensions: LogicalSize::new(1000, 1000),
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<VoidrayEngine>::run(options);
}
