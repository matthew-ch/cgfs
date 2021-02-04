use crate::primitives::*;
pub trait SceneObject {
    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(f32, Color)>;
}

pub struct Sphere {
    center: Point,
    radius: f32,
    color: Color,
}

impl Sphere {
    pub fn new(center: Point, radius: f32, color: Color) -> Self {
        Sphere {
            center,
            radius,
            color,
        }
    }
}

impl SceneObject for Sphere {
    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(f32, Color)> {
        let co: Vector = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * co.dot(&ray.direction);
        let c = co.dot(&co) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        if t1 >= t_min && t1 < t_max {
            return Some((t1, self.color));
        }
        if t2 >= t_min && t2 < t_max {
            return Some((t2, self.color));
        }
        return None
    }
}