#![allow(dead_code, unused_variables)]

use gui::engine_ui;
use voidray_common::{simple::Materials, Environments, Surfaces};
use voidray_launcher::gui_implementation::*;
use voidray_launcher::*;
use voidray_renderer::camera::Camera;
use voidray_renderer::color::*;
use voidray_renderer::preamble::*;
use voidray_renderer::render::post_process::PostProcessingData;
use voidray_renderer::render::renderer::Renderer;
use voidray_renderer::render::target::CpuRenderTarget;
use voidray_renderer::render::viewport::Viewport;
use voidray_renderer::scene::Scene;
use voidray_renderer::settings::Settings;
use voidray_renderer::settings::Tonemap;
use voidray_renderer::texture::SampleType;
use voidray_renderer::vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use voidray_renderer::vulkano::pipeline::graphics;
use voidray_renderer::vulkano::render_pass::Subpass;

mod gui;
mod widgets;
mod utils;

pub struct VoidrayEngine {
    pub target: Arc<CpuRenderTarget>,
    pub scene: Arc<RwLock<Scene>>,
    pub settings: Arc<RwLock<Settings>>,
    pub renderer: Renderer,
    viewport: Viewport,
}

impl Engine for VoidrayEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let api = context.api();
        let target = CpuRenderTarget::new(api.compute_queue(), 2, [500, 500]);
        let mut scene = Scene::empty();

        let red = scene.add_material(Materials::metal(hex_color(0xE78999), 0.05));
        let yellow = scene.add_material(Materials::dielectric(1.5));
        let green = scene.add_material(Materials::metal(hex_color(0xB3E7AA), 0.1));
        let blue = scene.add_material(Materials::metal(hex_color(0x7CA3E7), 0.01));
        let grey = scene.add_material(Materials::metal(hex_color(0xAAAAAA), 0.01));
        let diffuse = scene.add_material(Materials::lambertian(hex_color(0x7CA3E7)));
        let light_mtl = scene.add_material(Materials::colored_emissive(hex_color(0xFF0F0F), 200.0));

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

        let glass_2 = scene.add_material(Materials::dielectric(1.5));
        let sph = scene.add_analytic_surface(Surfaces::sphere(vec3!(1.5, 0.7, -3.1), 0.7));
        let sph_inner =
            scene.add_analytic_surface(Surfaces::sphere(vec3!(1.5, 0.695, -3.1), 0.695));
        scene.add_object(glass_2, sph);
        scene.add_object(diffuse, sph_inner);

        let light = scene.add_analytic_surface(Surfaces::sphere(vec3!(1.2, 8.0, -1.5), 2.0));
        scene.add_object(light_mtl, light);

        scene.camera = Camera::look_at(
            vec3!(0.7166, 2.8803, -9.2992),
            vec3!(0.8673, 0.9557, 0.2095),
            vec3!(0.0, 1.0, 0.0),
            0.6911,
        );
        scene.camera.dof = Some((0.12, vec3!(0.1, 0.6, -2.0)));

        let saloon_albedo = scene.add_image_texture("assets/wood_albedo.tif", SampleType::Bilinear);
        let saloon_normal = scene.add_image_texture("assets/wood_normal.tif", SampleType::Nearest);

        let gnd = scene.add_analytic_surface(Surfaces::ground_plane(0.0));
        let gnd_mat =
            scene.add_material(Materials::lambertian_texture(saloon_albedo, saloon_normal));
        scene.add_object(gnd_mat, gnd);

        scene.environment = Environments::hdri("assets/indoor.exr");

        let mut settings = Settings::default();
        settings.color_management.tonemap = Tonemap::ACES;
        settings.color_management.gamma = 1.0;
        settings.color_management.exposure = 2.0;

        let scene = Arc::new(RwLock::new(scene));
        let settings = Arc::new(RwLock::new(settings));

        Self {
            target: target.clone(),
            scene: scene.clone(),
            settings: settings.clone(),
            viewport: Viewport::new(
                api.graphics_queue(),
                api.compute_queue(),
                context.viewport_subpass(),
                target.clone(),
            ),
            renderer: Renderer::new(api.compute_queue(), scene, settings, target),
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
        let samples = self.renderer.samples();
        let mut scale = samples.1 as f32 / samples.0 as f32;
        if !scale.is_normal() {
            scale = 0.0;
        }

        let settings = self.settings.read().unwrap();
        let data = Some(PostProcessingData {
            scale,
            exposure: settings.color_management.exposure,
            gamma: settings.color_management.gamma,
            tonemap: settings.color_management.tonemap.as_i32(),
        });

        self.viewport.draw(command_buffer, viewport, data);
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1500, 1000),
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<VoidrayEngine>::run(options);
}
