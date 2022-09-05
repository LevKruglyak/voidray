use cgmath::InnerSpace;

use crate::{core::{Vec3, ray::{Hittable, Ray, HitRecord}, material::Material, object::Object, Float}, utils::{aabb::{Bounded, AABB}, color::Color}};

pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
    pub material: Box<dyn Material>,
}

impl Bounded for Sphere {
    fn bounds(&self) -> AABB {
        AABB {
            min: self.center - Vec3::new(self.radius, self.radius, self.radius),
            max: self.center + Vec3::new(self.radius, self.radius, self.radius),
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = ray.origin - self.center;

        let a = ray.direction.magnitude2();
        let half_b = oc.dot(ray.direction);
        let c = oc.magnitude2() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in an acceptable range
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = ray.at(root);
        let normal = (point - self.center) / self.radius;

        Some(HitRecord::new(point, normal, root, ray))
    }
}

impl Material for Sphere {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> (Color, Option<Ray>) {
        self.material.scatter(ray, hit)
    }
}

impl Object for Sphere {}
