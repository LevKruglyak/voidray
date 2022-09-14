use crate::examples::example_import::*;

pub fn scene() -> (Scene, Settings, [u32; 2]) {
    let mut scene = Scene::empty();
    let mut settings = Settings::default();

    let dimensions = [1000, 1000];

    let mushroom_albedo = scene.add_image_texture("assets/mushroom_albedo.jpg", SampleType::Bilinear);
    let mushroom_normal = scene.add_image_texture("assets/mushroom_normal.jpg", SampleType::Bilinear);

    let mushroom_mtl = scene.add_material(Materials::lambertian_texture(mushroom_albedo, mushroom_normal));
    let mushroom = scene.add_mesh_from_file("assets/mushroom.obj");
    scene.add_object(mushroom_mtl, mushroom);

    let ground_albedo = scene.add_image_texture("assets/mossy_ground_albedo.jpg", SampleType::Bilinear);
    let ground_normal = scene.add_image_texture("assets/mossy_ground_normal.jpg", SampleType::Bilinear);

    let ground_mtl = scene.add_material(Materials::lambertian_texture(ground_albedo, ground_normal));
    let ground = scene.add_mesh_from_file("assets/mossy_ground.obj");
    scene.add_object(ground_mtl, ground);

    // let sph = scene.add_analytic_surface(Surfaces::sphere(vec3!(5.0, 10.0, 0.0), 7.0));
    // let light = scene.add_material(Materials::emissive(15.0));
    // scene.add_object(light, sph);

    scene.camera.eye = vec3!(0.2, 2.8, -10.5);
    scene.camera.direction = vec3!(0.0, -0.2, 1.0);
    scene.camera.fov = 0.17;
    scene.camera.dof = Some((0.17, vec3!(0.06, 2.14, 0.18)));

    settings.color_management.gamma = 1.0;
    settings.color_management.exposure = 1.0;
    settings.color_management.tonemap = Tonemap::ACES;

    scene.environment = Environments::hdri("assets/studio.exr");

    (scene, settings, dimensions)
}
