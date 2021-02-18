mod components;
mod objects;
pub use components::*;
pub use objects::*;
use std::thread;
use std::mem;
use std::ops::RangeInclusive;

pub(crate) const EPS: f64 = 0.001;
pub(crate) const AIR_REFRACTION_INDEX: f64 = 1.0;

pub struct Canvas {
    width: u16,
    height: u16,
    image_data: Vec<ColorPack>,
}

impl Canvas {
    pub fn new(width: u16, height: u16, background: Color) -> Canvas {
        assert!(width > 0 && height > 0);

        let image_data = vec![background.clamp(); width as usize * height as usize];

        Canvas {
            width,
            height,
            image_data,
        }
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    fn index(&self, x: u16, y: u16) -> usize {
        assert!(x < self.width && y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
        let index = self.index(x, y);
        self.image_data[index] = color.clamp();
    }

    pub fn data(&self) -> &[u8] {
        let p = self.image_data.as_ptr();
        unsafe {
            std::slice::from_raw_parts(
                p as *const u8,
                self.image_data.len() * std::mem::size_of::<ColorPack>(),
            )
        }
    }

    pub fn render(&mut self, scene: &Scene, depth: u32, samples: u16) {
        let width = self.width;
        let height = self.height;
        for x in 0..width {
            for y in 0..height {
                self.super_sampling(x, y, samples, scene, depth);
            }
        }
    }

    pub fn render_mth(&mut self, scene: &Scene, threads: u16, depth: u32, samples: u16) {
        let mut v = Vec::new();
        for i in 0..threads {
            let scene = unsafe { Box::new(mem::transmute::<_, &'static Scene>(scene)) };
            let canvas = unsafe { Box::new(mem::transmute::<_, &'static mut Canvas>(&mut *self)) };
            let width = self.width;
            let height = self.height;
            v.push(thread::spawn(move || {
                for x in 0..width {
                    if x % threads != i {
                        continue;
                    }
                    for y in 0..height {
                        canvas.super_sampling(x, y, samples, &scene, depth);
                    }
                }
            }));
        }
        for t in v {
            t.join().unwrap();
        }
    }

    fn super_sampling(&mut self, x: u16, y: u16, n: u16, scene: &Scene, depth: u32) {
        let mut color = Color::black();
        for i in 0..n {
            for j in 0..n {
                let ray = scene.canvas_to_viewport(x * n + i, y * n + j, self.width * n, self.height * n);
                color = color + scene.trace_ray(&ray, AIR_REFRACTION_INDEX, 1.0..=f64::INFINITY, depth);
            }
        }
        self.set_pixel(x, y, color * (1. / (n * n) as f64));
    }
}

pub struct Scene {
    viewport_width: f64,
    viewport_height: f64,
    background: Color,
    objects: Vec<Box<dyn SceneObject + Sync>>,
    lights: Vec<Box<dyn LightObject + Sync>>,
    camera_position: Point,
    camera_rotation: Matrix,
    camera_distance: f64,
}

impl Scene {
    pub fn new(
        viewport_width: f64,
        viewport_height: f64,
        background: Color,
    ) -> Self {
        Scene {
            viewport_width,
            viewport_height,
            background,
            objects: Vec::new(),
            lights: Vec::new(),
            camera_position: Point::zero(),
            camera_rotation: Matrix::identity(),
            camera_distance: 1.,
        }
    }

    pub fn set_camera(&mut self, position: Point, rotation: Matrix, distance: f64) {
        self.camera_position = position;
        self.camera_rotation = rotation;
        self.camera_distance = distance;
    }

    pub fn add_object(&mut self, object: Box<dyn SceneObject + Sync>) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Box<dyn LightObject + Sync>) {
        self.lights.push(light);
    }

    pub fn canvas_to_viewport(&self, x: u16, y: u16, width: u16, height: u16) -> Ray {
        Ray {
            origin: self.camera_position,
            direction: self.camera_rotation.dot(&Vector {
                dx: (x as f64 / width as f64 - 0.5) * self.viewport_width,
                dy: (0.5 - y as f64 / height as f64) * self.viewport_height,
                dz: self.camera_distance,
            })
        }
    }

    fn compute_lighting(&self, point: &Point, normal: &Vector, view: &Vector, specular: i32) -> f64 {
        self.lights.iter().map(|light| light.intensity_from(self, point, normal, view, specular)).sum()
    }

    pub fn hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult> {
        let mut result: Option<HitTestResult> = None;

        for object in self.objects.iter() {
            if let Some(r) = object.hit_test(ray, &t_range) {
                if result.is_none() {
                    result = Some(r)
                } else if r.t < result.unwrap().t {
                    result = Some(r)
                }
            }
        }

        result
    }

    fn container_hit_test(&self, ray: &Ray, t_range: &RangeInclusive<f64>) -> Option<HitTestResult> {
        let mut result: Option<HitTestResult> = None;

        for object in self.objects.iter() {
            if let Some(r) = object.hit_test(ray, &t_range) {
                if r.normal.dot(&ray.direction) <= 0. {
                    continue;
                }
                if result.is_none() {
                    result = Some(r)
                } else if r.t < result.unwrap().t {
                    result = Some(r)
                }
            }
        }

        result
    }

    pub fn trace_ray(&self, ray: &Ray, refraction_index: f64, t_range: RangeInclusive<f64>, depth: u32) -> Color {
        let result = self.hit_test(ray, &t_range);

        result.map_or(self.background, |hit| {
            let opaque_color = {
                let local_color: Color = hit.material.color * self.compute_lighting(&hit.point, &hit.normal, &(-ray.direction), hit.material.specular);
                if depth == 0 || hit.material.reflective <= 0. {
                    local_color
                } else {
                    let reflected_ray = Ray {
                        origin: hit.point,
                        direction: hit.normal.reflect(&(-ray.direction))
                    };
                    let reflected_color = self.trace_ray(&reflected_ray, 1.0, EPS..=f64::INFINITY, depth - 1);
                    local_color * (1. - hit.material.reflective) + reflected_color * hit.material.reflective
                }
            };
            if depth == 0 || hit.material.transparency.is_none() {
                opaque_color
            } else {
                let in_vector = ray.direction / ray.direction.length();
                let going_outside_object = hit.normal.dot(&in_vector) > 0.;
                let new_refraction_index = if going_outside_object {
                    self.container_hit_test(&Ray { origin: hit.point, direction: ray.direction }, &(EPS..=f64::INFINITY))
                    .map(|container_hit| container_hit.material.transparency)
                    .flatten().unwrap_or(AIR_REFRACTION_INDEX)
                } else {
                    hit.material.transparency.unwrap()
                };
                let normal = if going_outside_object { -hit.normal } else { hit.normal };
                let cos = normal.dot(&in_vector);
                let k = refraction_index / new_refraction_index;
                let d = 1. - k * k * (1. - cos * cos);
                if d < 0. {
                    opaque_color
                } else {
                    let refraction_vector: Vector = (in_vector - normal * cos) * k - normal * d.sqrt();
                    let p = cos.abs().sqrt();
                    opaque_color * (1. - p) + self.trace_ray(&Ray { origin: hit.point, direction: refraction_vector }, new_refraction_index, EPS..=f64::INFINITY, depth - 1) * p
                }
            }
        })
    }
}
