use crate::examples::example_import::*;

pub fn scene() -> (Scene, Settings, [u32; 2]) {
    let mut scene = Scene::empty();
    let settings = Settings::default();
    let dimensions = [1000, 1000];

    let ground = scene.add_analytic_surface(Surfaces::ground_plane(0.15));
    // let uv_test = scene.add_image_texture("assets/uv_test.png", SampleType::Bilinear);
    // let ground_mat = scene.add_material(Materials::lambertian_texture_no_normal(uv_test));

    let ground_mat = scene.add_material(Materials::lambertian(hex_color(0x0F0F0F)));
    scene.add_object(ground_mat, ground);

    let monkey = scene.add_mesh_from_file("assets/material_monkey.obj");
    // let monkey_mtl = scene.add_material(Materials::metal(hex_color(0x7CA3E7), 0.05));
    let monkey_mtl = scene.add_material(Materials::dielectric(1.33));
    scene.add_object(monkey_mtl, monkey);

    scene.camera.eye = vec3!(2.9, 1.0, 11.0);
    scene.camera.direction = vec3!(-0.4, 0.0, -1.0);

    scene.environment = Environments::hdri("assets/studio.exr");
    // scene.environment = Environments::uniform(hex_color(0xA0A0A0));

    (scene, settings, dimensions)
}
