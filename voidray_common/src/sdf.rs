use std::sync::Arc;

use voidray_renderer::{
    aabb::{Bounded, AABB},
    ray::{HitRecord, Ray},
    traits::AnalyticSurface,
    vec3,
    vector::*,
};

pub trait SDF: Send + Sync {
    fn sdf(&self, point: Vec3) -> Float;

    // https://iquilezles.org/articles/normalsSDF
    fn normal(&self, point: Vec3) -> Vec3 {
        let e = Vec2::new(1.0, -1.0) * 0.5773 * 0.0005;
        let ex = e.x;
        let ey = e.y;
        (vec3!(ex, ey, ey) * self.sdf(point + vec3!(ex, ey, ey))
            + vec3!(ey, ey, ex) * self.sdf(point + vec3!(ey, ey, ex))
            + vec3!(ey, ex, ey) * self.sdf(point + vec3!(ey, ex, ey))
            + vec3!(ex) * self.sdf(point + vec3!(ex)))
        .normalize()
    }

    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        // Begin SDF trace
        let mut ro = ray.origin;
        let mut rd = ray.direction;

        // Make sure direction is normalized
        rd = rd.normalize();
        let mut distance;
        let mut cum_distance = 0.0;
        let mut count = 0;

        loop {
            if count > 100 {
                return None;
            }

            distance = self.sdf(ro);
            if !distance.is_finite() {
                return None;
            }
            cum_distance += distance;
            ro += rd * distance;

            if distance < t_min {
                // Hit
                let normal = self.normal(ro);
                return Some(HitRecord::new(
                    ro,
                    normal,
                    cum_distance,
                    Vec2::new(0.0, 0.0),
                    ray,
                ));
            }

            if distance > t_max {
                // Escaped
                return None;
            }

            count += 1;
        }
    }
}

pub struct SurfaceSDF {
    pub sdf: Arc<dyn SDF>,
}

impl Bounded for SurfaceSDF {
    fn bounds(&self) -> AABB {
        AABB::default()
    }
}

impl AnalyticSurface for SurfaceSDF {
    fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        self.sdf.hit(ray, t_min, t_max)
    }
}

pub struct SphereSDF {
    pub center: Vec3,
    pub radius: Float,
}

impl SDF for SphereSDF {
    fn sdf(&self, point: Vec3) -> Float {
        (self.center - point).magnitude() - self.radius
    }
}

pub struct MandelbulbSdf {}

impl SDF for MandelbulbSdf {
    fn sdf(&self, pos: Vec3) -> Float {
        let mut w = pos;
        let mut m = w.magnitude2();
        let mut dz = 1.0;

        for i in 0..1 {
            dz = 8.0 * m.powf(3.5) * dz + 1.0;

            let r = w.magnitude();
            let b = 8.0 * (w.y / r).acos();
            let a = 8.0 * w.x.atan2(w.z);

            w = pos + r.powf(8.0) * vec3!(b.sin() * a.sin(), b.cos(), b.sin() * a.cos());
            m = w.magnitude2();

            if m > 256.0 {
                break;
            }
        }

        0.25 * m.ln() * m.sqrt() / dz
    }
    //     vec3 w = p;
    //     float m = dot(w,w);
    //
    //     vec4 trap = vec4(abs(w),m);
    // 	float dz = 1.0;
    //
    // 	for( int i=0; i<4; i++ )
    //     {
    //         // trigonometric version (MUCH faster than polynomial)
    //
    //         // dz = 8*z^7*dz
    // 		dz = 8.0*pow(m,3.5)*dz + 1.0;
    //
    //         // z = z^8+c
    //         float r = length(w);
    //         float b = 8.0*acos( w.y/r);
    //         float a = 8.0*atan( w.x, w.z );
    //         w = p + pow(r,8.0) * vec3( sin(b)*sin(a), cos(b), sin(b)*cos(a) );
    // #endif
    //
    //         trap = min( trap, vec4(abs(w),m) );
    //
    //         m = dot(w,w);
    // 		if( m > 256.0 )
    //             break;
    //     }
    //
    //     resColor = vec4(m,trap.yzw);
    //
    //     // distance estimation (through the Hubbard-Douady potential)
    //     return 0.25*log(m)*sqrt(m)/dz;
}
