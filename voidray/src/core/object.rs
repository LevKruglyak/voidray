use crate::utils::aabb::Bounded;

use super::{ray::Hittable, material::Material};

pub trait Object: Sync + Hittable + Bounded + Material {}
