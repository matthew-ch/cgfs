use std::ops::RangeInclusive;
use crate::{EPS, Scene, components::*};

#[derive(Clone, Copy, Debug)]
pub struct HitTestResult {
    pub t: f64,
    pub point: Point,
    pub normal: Vector,
    pub material: Material,
}

pub trait SceneObject {
    fn hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult>;
}

pub struct SphereObject {
    pub sphere: Sphere,
    pub material: Material,
}

impl SceneObject for SphereObject {
    fn hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult> {
        let (t1, t2) = self.sphere.compute_ray_intersection(ray)?;
        for &t in [t1, t2].iter() {
            if t_range.contains(&t) {
                let point: Point = ray.origin + ray.direction * t;
                let normal: Vector = point - self.sphere.center;
                return Some(HitTestResult {
                    t,
                    point,
                    normal: normal / normal.length(),
                    material: self.material,
                });
            }
        }
        None
    }
}

pub enum BooleanOperation {
    UNION,
    INTERSECTION,
    SUBTRACTION
}

pub struct BooleanOperationSpheresObject {
    pub sphere_a: Sphere,
    pub operation: BooleanOperation,
    pub sphere_b: Sphere,
    pub material: Material,
}

impl SceneObject for BooleanOperationSpheresObject {
    fn hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult> {
        let ta = self.sphere_a.compute_ray_intersection(ray);
        let tb = self.sphere_b.compute_ray_intersection(ray);
        if ta.is_none() && tb.is_none() {
            return None;
        }
        let range_a = ta.unwrap_or((-f64::INFINITY, -f64::INFINITY));
        let range_b = tb.unwrap_or((-f64::INFINITY, -f64::INFINITY));
        let has_no_intersection = range_b.0 > range_a.1 || range_a.0 > range_b.1;
        let break_points: Vec<(f64, &Sphere)> = match self.operation {
            BooleanOperation::UNION => {
                if has_no_intersection {
                    if range_b.0 > range_a.1 {
                        vec![
                            (range_a.0, &self.sphere_a),
                            (range_a.1, &self.sphere_a),
                            (range_b.0, &self.sphere_b),
                            (range_b.1, &self.sphere_b),
                        ]
                    } else {
                        vec![
                            (range_b.0, &self.sphere_b),
                            (range_b.1, &self.sphere_b),
                            (range_a.0, &self.sphere_a),
                            (range_a.1, &self.sphere_a),
                        ]
                    }

                } else {
                    vec![
                        if range_a.0 < range_b.0 { (range_a.0, &self.sphere_a) } else { (range_b.0, &self.sphere_b) },
                        if range_a.1 < range_b.1 { (range_b.1, &self.sphere_b) } else { (range_a.1, &self.sphere_a) },
                    ]
                }
            },
            BooleanOperation::INTERSECTION => {
                if has_no_intersection {
                    Vec::new()
                } else {
                    vec![
                        if range_a.0 < range_b.0 { (range_b.0, &self.sphere_b) } else { (range_a.0, &self.sphere_a) },
                        if range_a.1 < range_b.1 { (range_a.1, &self.sphere_a) } else { (range_b.1, &self.sphere_b) },
                    ]
                }
            },
            BooleanOperation::SUBTRACTION => {
                if has_no_intersection {
                    vec![
                        (range_a.0, &self.sphere_a),
                        (range_a.1, &self.sphere_a),
                    ]
                } else if range_b.0 > range_a.0 {
                    if range_b.1 >= range_a.1 {
                        vec![
                            (range_a.0, &self.sphere_a),
                            (range_b.0, &self.sphere_b),
                        ]
                    } else {
                        vec![
                            (range_a.0, &self.sphere_a),
                            (range_b.0, &self.sphere_b),
                            (range_b.1, &self.sphere_a),
                            (range_a.1, &self.sphere_a),
                        ]
                    }
                } else if range_b.1 < range_a.1 {
                    vec![
                        (range_b.1, &self.sphere_b),
                        (range_a.1, &self.sphere_a),
                    ]
                } else {
                    Vec::new()
                }
            },
        };

        for (i, b) in break_points.into_iter().enumerate() {
            if t_range.contains(&b.0) {
                let point = ray.origin + ray.direction * b.0;
                let mut normal: Vector = point - b.1.center;
                let flip = {
                    let f = normal.dot(&ray.direction);
                    if i % 2 == 1 { f < 0. } else { f > 0. }
                };
                if flip {
                    normal = -normal;
                }
                return Some(HitTestResult {
                    t: b.0,
                    point,
                    normal: normal / normal.length(),
                    material: self.material,
                })
            }
        }

        None
    }
}

pub struct PolyhedronObject {
    pub triangles: Vec<Triangle>,
    pub material: Material,
}

impl SceneObject for PolyhedronObject {
    fn hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult> {
        let mut hits = self.triangles.iter()
            .flat_map(|triangle| triangle.compute_ray_intersection(ray).map(|t| (t, triangle.normal)))
            .collect::<Vec<_>>();
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for hit in hits {
            if t_range.contains(&hit.0) {
                let point: Point = ray.origin + ray.direction * hit.0;
                return Some(HitTestResult{
                    t: hit.0,
                    point,
                    normal: hit.1,
                    material: self.material,
                });
            }
        }
        None
    }
}

pub trait LightObject {
    fn intensity_from(&self, scene: &Scene, point: &Point, normal: &Vector, view: &Vector, specular: i32) -> f64;
}

fn compute_light_factor(normal: &Vector, light: &Vector, view: &Vector, specular: i32) -> f64 {
    normal.cos(light).max(0.) + if specular >= 0 {
        normal.reflect(light).cos(view).max(0.).powi(specular)
    } else {
        0.
    }
}

pub struct AmbientLight {
    pub intensity: f64,
}

impl LightObject for AmbientLight {
    fn intensity_from(&self, _scene: &Scene, _point: &Point, _normal: &Vector, _view: &Vector, _specular: i32) -> f64 {
        self.intensity
    }
}

pub struct PointLight {
    pub intensity: f64,
    pub position: Point,
}

impl LightObject for PointLight {
    fn intensity_from(&self, scene: &Scene, point: &Point, normal: &Vector, view: &Vector, specular: i32) -> f64 {
        let light: Vector = self.position - *point;
        if scene.hit_test(&Ray { origin: *point, direction: light }, &(EPS..=1.0)).is_some() {
            0.
        } else {
            self.intensity * compute_light_factor(normal, &light, view, specular)
        }
    }
}

pub struct DirectionalLight {
    pub intensity: f64,
    pub direction: Vector,
}

impl LightObject for DirectionalLight {
    fn intensity_from(&self, scene: &Scene, point: &Point, normal: &Vector, view: &Vector, specular: i32) -> f64 {
        if scene.hit_test(&Ray { origin: *point, direction: self.direction }, &(EPS..=f64::INFINITY)).is_some() {
            0.
        } else {
            self.intensity * compute_light_factor(normal, &self.direction, view, specular)
        }
    }
}


pub struct SceneModel {
    pub name: String,
    pub vertices: Vec<Point>,
    pub triangles: Vec<([usize; 3], Color)>,
    bounding_sphere: Option<Sphere>,
}

impl SceneModel {
    pub fn new(name: String, vertices: Vec<Point>, triangles: Vec<([usize; 3], Color)>) -> Self {
        Self {
            name,
            vertices,
            triangles,
            bounding_sphere: None,
        }
    }

    pub fn create_sphere_model(name: String, divides: usize, color: Color) -> Self {
        let mut vertices = vec![
            Point::from((0., 1., 0.)),
            Point::from((0., -1., 0.)),
        ];
        let mut triangles = Vec::new();
        let s = 2 * divides + 2;
        let step_h = (360. / s as f64).to_radians();
        let rxz_y_lookup = {
            let step_v = (180. / (divides + 1) as f64).to_radians();
            (1..=divides).into_iter().map(|d| (d as f64 * step_v).sin_cos()).collect::<Vec<_>>()
        };
        let mut lngs = Vec::new();
        for k in 0..s {
            let mut lng = Vec::new();
            let (sin, cos) = (step_h * k as f64).sin_cos();
            lng.push(0);
            for i in 0..divides {
                lng.push(vertices.len());
                vertices.push(Point::from((rxz_y_lookup[i].0 * cos, rxz_y_lookup[i].1, rxz_y_lookup[i].0 * sin)));
            }
            lng.push(1);
            lngs.push(lng);
        }
        for k in 0..s {
            let lng1 = &lngs[k];
            let lng2 = &lngs[(k + 1) % s];
            for i in 0..divides {
                triangles.push(([lng1[i], lng2[i+1], lng1[i+1]], color));
                triangles.push(([lng1[i+1], lng2[i+1], lng2[i+2]], color));
            }
        }

        SceneModel {
            name,
            triangles,
            vertices,
            bounding_sphere: Some(Sphere{ center: (0., 0., 0.).into(), radius: 1. }),
        }
    }

    pub fn get_bounding_sphere(&mut self) -> Sphere {
        if let Some(sphere) = self.bounding_sphere {
            sphere
        } else {
            let center: Point = self.vertices.iter().map(|&v| Point::from(v)).fold(Point::from((0., 0., 0., 0.)), |acc, v| acc + v) / self.vertices.len() as f64;
            let radius = self.vertices.iter().map(|&v| (Point::from(v) - center).length()).max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap_or(0.);
            let sphere = Sphere {
                center,
                radius,
            };
            self.bounding_sphere = Some(sphere);
            sphere
        }
    }
}

pub struct SceneModelInstance {
    pub model_name: String,
    pub transform: Matrix,
}