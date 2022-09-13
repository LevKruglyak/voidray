use voidray_common::simple::Materials;
use voidray_common::Environments;
use voidray_common::Surfaces;
use voidray_renderer::camera::Camera;
use voidray_renderer::color::*;
use voidray_renderer::preamble::*;
use voidray_renderer::scene::Scene;
use voidray_renderer::settings::Settings;
use voidray_renderer::settings::Tonemap;
use voidray_renderer::texture::SampleType;

pub fn scene() -> (Scene, Settings, [u32; 2]) {
    let mut scene = Scene::empty();
    let mut settings = Settings::default();
    let dimensions = [1000, 1000];

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
    let sph_inner = scene.add_analytic_surface(Surfaces::sphere(vec3!(1.5, 0.695, -3.1), 0.695));
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
    let gnd_mat = scene.add_material(Materials::lambertian_texture(saloon_albedo, saloon_normal));
    scene.add_object(gnd_mat, gnd);

    scene.environment = Environments::hdri("assets/indoor.exr");

    settings.color_management.tonemap = Tonemap::ACES;
    settings.color_management.gamma = 1.0;
    settings.color_management.exposure = 2.0;

    (scene, settings, dimensions)
}
