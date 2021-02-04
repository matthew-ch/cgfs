use crate::primitives::*;

#[derive(Clone, Copy, Debug)]
pub struct HitTestResult {
    pub t: f32,
    pub color: Color,
    pub point: Point,
    pub normal: Vector,
}

pub trait SceneObject {
    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitTestResult>;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
}

impl SceneObject for Sphere {
    fn hit_test(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitTestResult> {
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
        let st = if t1 >= t_min && t1 < t_max {
            Some(t1)
        } else if t2 >= t_min && t2 < t_max {
            Some(t2)
        } else {
            None
        };
        st.map(|t| {
            let point: Point = ray.origin + ray.direction * t;
            let normal: Vector = point - self.center;
            HitTestResult {
                t,
                color: self.color,
                point,
                normal: normal / normal.length(),
            }
        })
    }
}

pub trait LightObject {
    fn intensity_from(&self, point: &Point, normal: &Vector) -> f32;
}

pub struct AmbientLight {
    pub intensity: f32,
}

impl LightObject for AmbientLight {
    fn intensity_from(&self, _point: &Point, _normal: &Vector) -> f32 {
        self.intensity
    }
}

pub struct PointLight {
    pub intensity: f32,
    pub position: Point,
}

impl LightObject for PointLight {
    fn intensity_from(&self, point: &Point, normal: &Vector) -> f32 {
        let l: Vector = self.position - *point;
        normal.cos(&l).max(0.) * self.intensity
    }
}

pub struct DirectionalLight {
    pub intensity: f32,
    pub direction: Vector,
}

impl LightObject for DirectionalLight {
    fn intensity_from(&self, _point: &Point, normal: &Vector) -> f32 {
        let l = self.direction;
        normal.cos(&l).max(0.) * self.intensity
    }
}