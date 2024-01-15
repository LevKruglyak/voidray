use std::f32::EPSILON;
use std::fs::File;
use std::io::BufReader;

use crate::aabb::*;
use crate::bvh::*;
use crate::math::*;
use crate::ray::*;
use crate::vec3;
use crate::vector::*;
use obj::*;

#[derive(Debug)]
pub struct Triangle {
    vertices: [u32; 3],
    normal: Vec3,
}

#[derive(Debug)]
pub struct Vertex {
    position: Vec3,
    uv: Vec2,
    normal: Vec3,
}

impl Vertex {
    pub fn position(position: Vec3) -> Vertex {
        Vertex {
            position,
            uv: Vec2::new(0.0, 0.0),
            normal: vec3!(0.0),
        }
    }
}

pub struct Mesh {
    /// Vertex buffer
    vertices: Vec<Vertex>,
    bounds: AABB,
    triangles: Vec<Triangle>,
    bvh_root: BvhNode,
}

static SMALL_MESH: usize = 4;

impl Mesh {
    pub fn from_file(path: &str) -> Self {
        let obj: Obj<obj::TexturedVertex, u32> =
            load_obj(BufReader::new(File::open(path).unwrap())).unwrap();
        let mut vertices: Vec<Vertex> = Vec::new();

        for vertex in obj.vertices {
            vertices.push(Vertex {
                position: Vec3::new(
                    vertex.position[0] as Float,
                    vertex.position[1] as Float,
                    vertex.position[2] as Float,
                ),
                uv: Vec2::new(vertex.texture[0] as Float, vertex.texture[1] as Float),
                normal: Vec3::new(
                    vertex.normal[0] as Float,
                    vertex.normal[1] as Float,
                    vertex.normal[2] as Float,
                ),
            });
        }

        println!(
            "loaded '{}' with {} verticies, {} faces",
            path,
            vertices.len(),
            obj.indices.len() / 3
        );
        Self::from_buffers(vertices, obj.indices)
    }

    pub fn from_buffers(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let mut triangles = Vec::<Triangle>::new();

        for triangle in indices.chunks_exact(3) {
            let e1 =
                vertices[triangle[0] as usize].position - vertices[triangle[1] as usize].position;
            let e2 =
                vertices[triangle[2] as usize].position - vertices[triangle[1] as usize].position;
            let normal = e2.cross(e1).normalize();

            triangles.push(Triangle {
                vertices: [triangle[0], triangle[1], triangle[2]],
                normal,
            })
        }

        // Calculate bounding box
        let mut bounds = AABB::default();
        for vertex in &vertices[..] {
            bounds = AABB::surround(
                bounds,
                AABB {
                    min: vertex.position,
                    max: vertex.position,
                },
            );
        }

        let mut result = Self {
            vertices,
            triangles,
            bvh_root: BvhNode::None,
            bounds,
        };

        // Small mesh optimizations
        if indices.len() > SMALL_MESH * 3 {
            result.build_bvh();
        }
        result
    }

    pub fn build_bvh(&mut self) {
        self.bvh_root = BvhNode::from_list(&mut self.objects(), self);
    }

    pub fn hit(&self, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        if self.bvh_root != BvhNode::None {
            self.bvh_root.hit(ray, t_min, t_max, self).map(|a| a.0)
        } else {
            let mut result = None;
            let mut closest_so_far = t_max;

            for triangle in &self.triangles {
                if let Some(hit) = triangle.hit(ray, t_min, closest_so_far, self) {
                    if closest_so_far > hit.t {
                        closest_so_far = hit.t;
                        result = Some(hit);
                    }
                }
            }

            result
        }
    }
}

impl Triangle {
    fn hit(&self, ray: &Ray, tmin: Float, _: Float, mesh: &Mesh) -> Option<HitRecord> {
        let v0 = &mesh.vertices[self.vertices[0] as usize];
        let v1 = &mesh.vertices[self.vertices[1] as usize];
        let v2 = &mesh.vertices[self.vertices[2] as usize];

        // Edges
        let e1 = v1.position - v0.position;
        let e2 = v2.position - v0.position;

        let h = ray.direction.cross(e2);
        let a = e1.dot(h);

        if a > -tmin && a < tmin {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - v0.position;
        let u = f * s.dot(h);

        #[allow(clippy::manual_range_contains)]
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(e1);
        let v = f * ray.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * e2.dot(q);
        let mut normal = u * v1.normal + v * v2.normal + (1.0 - u - v) * v0.normal;
        let uv = u * v1.uv + v * v2.uv + (1.0 - u - v) * v0.uv;

        if normal.angle(self.normal).0 > degrees_to_radians(30.0) {
            normal = self.normal;
        }

        if t > tmin {
            Some(HitRecord::new(ray.at(t), normal, t, uv, ray))
        } else {
            None
        }
    }
}

/// Make it possible to create Bvh tree for mesh
impl BoundsCollection for Mesh {
    fn bounds_ref(&self, handle: usize) -> AABB {
        let triangle = &self.triangles[handle as usize];
        AABB::epsilon_expand(
            AABB::surround(
                AABB::from_point(self.vertices[triangle.vertices[0] as usize].position),
                AABB::surround(
                    AABB::from_point(self.vertices[triangle.vertices[1] as usize].position),
                    AABB::from_point(self.vertices[triangle.vertices[2] as usize].position),
                ),
            ),
            EPSILON,
        )
    }

    fn objects(&self) -> Vec<usize> {
        self.triangles
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect()
    }

    fn hit(&self, handle: usize, ray: &Ray, tmin: Float, tmax: Float) -> Option<HitRecord> {
        self.triangles[handle as usize].hit(ray, tmin, tmax, self)
    }
}

impl Bounded for Mesh {
    fn bounds(&self) -> AABB {
        self.bounds.clone()
    }
}
