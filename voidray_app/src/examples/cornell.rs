use voidray_renderer::math::degrees_to_radians;

use crate::examples::example_import::*;

pub fn scene() -> (Scene, Settings, [u32; 2]) {
    let mut scene = Scene::empty();
    let mut settings = Settings::default();

    settings.color_management.gamma = 1.0;
    settings.color_management.exposure = 2.0;
    settings.color_management.tonemap = Tonemap::Filmic;

    let dimensions = [500, 500];

    let red = scene.add_material(Materials::lambertian(Color(vec3!(0.65, 0.05, 0.05))));
    let white = scene.add_material(Materials::lambertian(Color(vec3!(0.73, 0.73, 0.73))));
    let green = scene.add_material(Materials::lambertian(Color(vec3!(0.12, 0.45, 0.15))));
    let light = scene.add_material(Materials::emissive(15.0));

    let floor = scene.add_mesh(Surfaces::quad(
            vec3!(0.0, 0.0, 0.0), 
            vec3!(0.0, 0.0, 555.0),
            vec3!(555.0, 0.0, 555.0),
            vec3!(555.0, 0.0, 0.0),
    ));
    let red_wall = scene.add_mesh(Surfaces::quad(
            vec3!(0.0, 0.0, 0.0), 
            vec3!(0.0, 0.0, 555.0),
            vec3!(0.0, 555.0, 555.0),
            vec3!(0.0, 555.0, 0.0),
    ));
    let green_wall = scene.add_mesh(Surfaces::quad(
            vec3!(555.0, 0.0, 0.0), 
            vec3!(555.0, 0.0, 555.0),
            vec3!(555.0, 555.0, 555.0),
            vec3!(555.0, 555.0, 0.0),
    ));
    let back_wall = scene.add_mesh(Surfaces::quad(
            vec3!(0.0, 0.0, 555.0), 
            vec3!(555.0, 0.0, 555.0),
            vec3!(555.0, 555.0, 555.0),
            vec3!(0.0, 555.0, 555.0),
    ));
    let ceil = scene.add_mesh(Surfaces::quad(
            vec3!(0.0, 555.0, 0.0), 
            vec3!(0.0, 555.0, 555.0),
            vec3!(555.0, 555.0, 555.0),
            vec3!(555.0, 555.0, 0.0),
    ));
    let light_plane = scene.add_mesh(Surfaces::quad(
            vec3!(213.0, 554.0, 227.0), 
            vec3!(213.0, 554.0, 332.0),
            vec3!(343.0, 554.0, 332.0),
            vec3!(343.0, 554.0, 227.0),
    ));

    scene.add_object(white, floor);
    scene.add_object(green, green_wall);
    scene.add_object(red, red_wall);
    scene.add_object(white, back_wall);
    scene.add_object(white, ceil);
    scene.add_object(light, light_plane);

    let sph = scene.add_analytic_surface(Surfaces::sphere(vec3!(555.0/2.0, 100.0, 555.0/2.0), 100.0));
    let glass = scene.add_material(Materials::dielectric(1.33));
    let sph_inner = scene.add_analytic_surface(Surfaces::sphere(vec3!(555.0/2.0, 100.0, 555.0/2.0), 99.9));
    let glass_inner = scene.add_material(Materials::lambertian(hex_color(0x0F1BF0)));
    scene.add_object(glass, sph);
    scene.add_object(glass_inner, sph_inner);

    scene.camera = Camera::look_at(
        vec3!(278.0, 278.0, -800.0), 
        vec3!(278.0, 278.0, 0.0), 
        vec3!(0.0, 1.0, 0.0), 
        degrees_to_radians(40.0));

    (scene, settings, dimensions)
}
