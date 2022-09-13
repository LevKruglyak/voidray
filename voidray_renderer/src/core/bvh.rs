use crate::aabb::*;
use crate::ray::*;
use crate::vector::*;

/// Acceleration strcture for faster ray-scene intersections
pub struct BvhTree<'s, S> {
    scene: &'s S,
    root: BvhNode,
}

pub trait BoundsCollection: Sync {
    fn hit(&self, handle: usize, ray: &Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
    fn bounds(&self, handle: usize) -> AABB;
    fn objects(&self) -> Vec<usize>;
}

impl<'s, S> BvhTree<'s, S>
where
    S: BoundsCollection,
{
    /// Build a Bvh tree from a scene
    pub fn build(scene: &'s S) -> Self {
        Self {
            scene,
            root: BvhNode::from_list(&mut scene.objects(), scene),
        }
    }

    fn hit(&self, ray: &Ray, tmin: Float, tmax: Float) -> Option<(HitRecord, usize)> {
        self.root.hit(ray, tmin, tmax, self.scene)
    }
}

pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(PartialEq)]
pub enum BvhNode {
    None,
    Object(usize),
    Split(AABB, Box<BvhNode>, Box<BvhNode>),
}

impl BvhNode {
    pub fn from_list<S>(objects: &mut Vec<usize>, scene: &S) -> Self
    where
        S: BoundsCollection,
    {
        match objects.len() {
            0 => BvhNode::None,
            1 => BvhNode::Object(objects[0]),
            _ => {
                // Calculate bounds for the whole node
                let bounds = objects
                    .iter()
                    .map(|object| scene.bounds(*object))
                    .reduce(AABB::surround)
                    .unwrap_or_default();

                // See which axis has the greatest variance among centroids
                let centroids = objects
                    .iter()
                    .map(|object| AABB::from_point(scene.bounds(*object).centroid()))
                    .reduce(AABB::surround)
                    .unwrap_or_default();

                let spread = centroids.max - centroids.min;
                let axis = if spread.x > spread.y && spread.x > spread.z {
                    Axis::X
                } else if spread.y > spread.x && spread.y > spread.z {
                    Axis::Y
                } else {
                    Axis::Z
                };

                // Sort objects by laying bounding boxes along an axis
                match axis {
                    Axis::X => {
                        objects.sort_by(|a, b| {
                            scene
                                .bounds(*a)
                                .centroid()
                                .x
                                .partial_cmp(&scene.bounds(*b).centroid().x)
                                .unwrap()
                        });
                    }
                    Axis::Y => {
                        objects.sort_by(|a, b| {
                            scene
                                .bounds(*a)
                                .centroid()
                                .y
                                .partial_cmp(&scene.bounds(*b).centroid().y)
                                .unwrap()
                        });
                    }
                    Axis::Z => {
                        objects.sort_by(|a, b| {
                            scene
                                .bounds(*a)
                                .centroid()
                                .z
                                .partial_cmp(&scene.bounds(*b).centroid().z)
                                .unwrap()
                        });
                    }
                }

                // Assign first half to left half and second half to right half
                let mut left_list = Vec::new();
                let mut right_list = Vec::new();

                for (index, element) in objects.iter().enumerate() {
                    if index < objects.len() / 2 {
                        left_list.push(*element);
                    } else {
                        right_list.push(*element);
                    }
                }

                // Create the node
                BvhNode::Split(
                    bounds,
                    Box::new(BvhNode::from_list(&mut left_list, scene)),
                    Box::new(BvhNode::from_list(&mut right_list, scene)),
                )
            }
        }
    }

    pub fn hit<S>(
        &self,
        ray: &Ray,
        t_min: Float,
        t_max: Float,
        scene: &S,
    ) -> Option<(HitRecord, usize)>
    where
        S: BoundsCollection,
    {
        match self {
            BvhNode::Object(handle) => {
                return scene
                    .hit(*handle, ray, t_min, t_max)
                    .map(|hit| (hit, *handle));
            }
            BvhNode::Split(bounds, left, right) => {
                if bounds.hit(ray, t_min, t_max) {
                    let hit_left = left.hit(ray, t_min, t_max, scene);
                    let hit_right = right.hit(ray, t_min, t_max, scene);

                    return merge_optionals(hit_left, hit_right);
                }
            }
            _ => {}
        }

        None
    }
}

/// Commonly used to merge hit record results in a Bvh tree
#[inline]
fn merge_optionals<H>(hit_left: Option<H>, hit_right: Option<H>) -> Option<H>
where
    H: PartialOrd,
{
    match (hit_left, hit_right) {
        (Some(record_left), Some(record_right)) => {
            if record_left < record_right {
                Some(record_left)
            } else {
                Some(record_right)
            }
        }
        (Some(record_left), None) => Some(record_left),
        (None, Some(record_right)) => Some(record_right),
        _ => None,
    }
}
