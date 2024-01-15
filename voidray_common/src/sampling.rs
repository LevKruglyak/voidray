use voidray_renderer::{
    rand::{
        rand_distr::{UnitDisc},
        Distribution,
    },
    vec3,
    vector::{Float, Vec3},
};

pub struct UnitHemisphere;

impl Distribution<Vec3> for UnitHemisphere {
    fn sample<R: voidray_renderer::rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let d = rng.sample(UnitDisc);
        let z = (0.0 as Float)
            .max(1.0 as Float - d[0] * d[0] - d[1] * d[1])
            .sqrt();
        vec3!(d[0], d[1], z)
    }
}
