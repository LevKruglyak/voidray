use crate::examples::example_import::*;

pub fn scene() -> (Scene, Settings, [u32; 2]) {
    let mut scene = Scene::empty();
    let settings = Settings::default();
    let dimensions = [1000, 1000];

    let ground = scene.add_analytic_surface(Surfaces::ground_plane(0.0));
    let uv_test = scene.add_image_texture("assets/uv_test.png", SampleType::Nearest);
    let ground_mat = scene.add_material(Materials::lambertian_texture_no_normal(uv_test,));

    scene.add_object(ground_mat, ground);

    let material_stand = scene.add_mesh_from_file("assets/material_testing_stand.obj");
    let stand_mtl = scene.add_material(Materials::lambertian(hex_color(0x0F0F0F)));
    scene.add_object(stand_mtl, material_stand);

    let material_main = scene.add_mesh_from_file("assets/material_testing_main.obj");
    let main_mtl = scene.add_material(Materials::lambertian(hex_color(0xFF0F0F)));
    scene.add_object(main_mtl, material_main);

    scene.camera.eye = vec3!(0.5, 0.6, 5.0);

    scene.environment = Environments::hdri("assets/studio.exr");

    (scene, settings, dimensions)
}
