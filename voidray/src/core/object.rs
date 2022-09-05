use crate::utils::aabb::Bounded;

use super::{material::Material, ray::Hittable};

pub trait Object: Sync + Hittable + Bounded + Material {}
