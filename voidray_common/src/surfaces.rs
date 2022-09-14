use voidray_renderer::aabb::*;
use voidray_renderer::mesh::{Mesh, Vertex};
use voidray_renderer::preamble::*;
use voidray_renderer::ray::*;
use voidray_renderer::traits::*;

pub struct Surfaces {}

impl Surfaces {
    pub fn sphere(center: Vec3, radius: Float) -> Arc<dyn AnalyticSurface> {
        Arc::new(Sphere { center, radius })
    }

    pub fn ground_plane(height: Float) -> Arc<dyn AnalyticSurface> {
        Arc::new(GroundPlane { height })
    }

    pub fn quad(q1: Vec3, q2: Vec3, q3: Vec3, q4: Vec3) -> Arc<Mesh> {
        let vertices = vec![
            Vertex::position(q1),
            Vertex::position(q2),
            Vertex::position(q3),
            Vertex::position(q4),
        ];
        let indices = vec![0, 1, 2, 2, 0, 3,];
        
        Arc::new(Mesh::from_buffers(vertices, indices))
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
}

impl Bounded for Sphere {
    fn bounds(&self) -> AABB {
        AABB {
            min: self.center - Vec3::new(self.radius, self.radius, self.radius),
            max: self.center + Vec3::new(self.radius, self.radius, self.radius),
        }
    }
}

impl AnalyticSurface for Sphere {
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

        Some(HitRecord::new(
            point,
            normal,
            root,
            Vec2::new(0.0, 0.0),
            ray,
        ))
    }
}

pub struct GroundPlane {
    pub height: Float,
}

impl AnalyticSurface for GroundPlane {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let t = (self.height - ray.origin.y) / ray.direction.y;

        if t <= t_min || t >= t_max {
            return None;
        }

        let world_pos = ray.at(t);
        let uv = Vec2::new(world_pos.x, world_pos.z);

        Some(HitRecord::new(
            world_pos,
            Vec3::new(0.0, 1.0, 0.0),
            t,
            uv,
            ray,
        ))
    }
}

impl Bounded for GroundPlane {
    fn bounds(&self) -> AABB {
        AABB {
            min: Vec3::new(-INF, self.height - 0.0001, -INF),
            max: Vec3::new(INF, self.height + 0.0001, INF),
        }
    }
}
