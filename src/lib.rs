mod primitives;
mod objects;
pub use primitives::*;
pub use objects::*;
use std::thread;
use std::mem;

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

    pub fn render(&mut self, scene: &Scene, depth: u32) {
        let o = Point::default();
        let width = self.width;
        let height = self.height;
        for x in 0..width {
            for y in 0..height {
                let d = scene.canvas_to_viewport(x, y, width, height);
                let ray = Ray { origin: o, direction: d - o };
                self.set_pixel(x, y, scene.trace_ray(&ray, 1.0, f64::INFINITY, depth));
            }
        }
    }

    pub fn render_mth(&mut self, scene: &Scene, depth: u32) {
        let o = Point::default();
        let mut v = Vec::new();
        for i in 0..4 {
            let scene = unsafe { Box::new(mem::transmute::<_, &'static Scene>(scene)) };
            let canvas = unsafe { Box::new(mem::transmute::<_, &'static mut Canvas>(&mut *self)) };
            let width = self.width;
            let height = self.height;
            v.push(thread::spawn(move || {
                for x in 0..width {
                    if x % 4 != i {
                        continue;
                    }
                    for y in 0..height {
                        let d = scene.canvas_to_viewport(x, y, width, height);
                        let ray = Ray { origin: o, direction: d - o };
                        canvas.set_pixel(x, y, scene.trace_ray(&ray, 1.0, f64::INFINITY, depth));
                    }
                }
            }));
        }
        for t in v {
            t.join().unwrap();
        }
    }
}

pub struct Scene {
    viewport_width: f64,
    viewport_height: f64,
    distance_to_camera: f64,
    background: Color,
    objects: Vec<Box<dyn SceneObject + Sync>>,
    lights: Vec<Box<dyn LightObject + Sync>>,
}

impl Scene {
    pub fn new(
        viewport_width: f64,
        viewport_height: f64,
        distance_to_camera: f64,
        background: Color,
    ) -> Self {
        Scene {
            viewport_width,
            viewport_height,
            distance_to_camera,
            background,
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Box<dyn SceneObject + Sync>) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Box<dyn LightObject + Sync>) {
        self.lights.push(light);
    }

    pub fn canvas_to_viewport(&self, x: u16, y: u16, width: u16, height: u16) -> Point {
        Point {
            x: (x as f64 / width as f64 - 0.5) * self.viewport_width,
            y: (0.5 - y as f64 / height as f64) * self.viewport_height,
            z: self.distance_to_camera,
        }
    }

    fn compute_lighting(&self, point: &Point, normal: &Vector, view: &Vector, specular: i32) -> f64 {
        self.lights.iter().map(|light| light.intensity_from(self, point, normal, view, specular)).sum()
    }

    pub fn hit_test(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitTestResult> {
        let mut result: Option<HitTestResult> = None;

        for object in self.objects.iter() {
            if let Some(r) = object.hit_test(ray, t_min, t_max) {
                if result.is_none() {
                    result = Some(r)
                } else if r.t < result.unwrap().t {
                    result = Some(r)
                }
            }
        }

        result
    }

    pub fn trace_ray(&self, ray: &Ray, t_min: f64, t_max: f64, depth: u32) -> Color {
        let result = self.hit_test(ray, t_min, t_max);

        result.map_or(self.background, |hit| {
            let local_color: Color = hit.color * self.compute_lighting(&hit.point, &hit.normal, &(-ray.direction), hit.specular);
            if depth == 0 || hit.reflective <= 0. {
                local_color
            } else {
                let reflected_ray = Ray {
                    origin: hit.point,
                    direction: hit.normal.reflect(&(-ray.direction))
                };
                let reflected_color = self.trace_ray(&reflected_ray, 0.001, f64::INFINITY, depth - 1);
                local_color * (1. - hit.reflective) + reflected_color * hit.reflective
            }
        })
    }
}
