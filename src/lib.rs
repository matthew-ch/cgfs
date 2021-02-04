mod primitives;
mod objects;
pub use primitives::*;
pub use objects::*;
use std::thread;
use std::mem;

pub struct Canvas {
    width: u16,
    height: u16,
    image_data: Vec<Color>,
}

impl Canvas {
    pub fn new(width: u16, height: u16, background: Color) -> Canvas {
        assert!(width > 0 && height > 0);

        let image_data = vec![background; width as usize * height as usize];

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
        self.image_data[index] = color;
    }

    pub fn data(&self) -> &[u8] {
        let p = self.image_data.as_ptr();
        unsafe {
            std::slice::from_raw_parts(
                p as *const u8,
                self.image_data.len() * std::mem::size_of::<Color>(),
            )
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let o = Point::default();
        let width = self.width;
        let height = self.height;
        for x in 0..width {
            for y in 0..height {
                let d = scene.canvas_to_viewport(x, y, width, height);
                let ray = Ray { origin: o, direction: d - o };
                self.set_pixel(x, y, scene.trace_ray(&ray, 1.0, f32::MAX));
            }
        }
    }

    pub fn render_mth(&mut self, scene: &Scene) {
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
                        canvas.set_pixel(x, y, scene.trace_ray(&ray, 1.0, f32::MAX));
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
    viewport_width: f32,
    viewport_height: f32,
    distance_to_camera: f32,
    background: Color,
    objects: Vec<Box<dyn SceneObject + Sync>>,
}

impl Scene {
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
        distance_to_camera: f32,
        background: Color,
    ) -> Self {
        Scene {
            viewport_width,
            viewport_height,
            distance_to_camera,
            background,
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Box<dyn SceneObject + Sync>) {
        self.objects.push(object);
    }

    pub fn canvas_to_viewport(&self, x: u16, y: u16, width: u16, height: u16) -> Point {
        Point {
            x: (x as f32 / width as f32 - 0.5) * self.viewport_width,
            y: (0.5 - y as f32 / height as f32) * self.viewport_height,
            z: self.distance_to_camera,
        }
    }

    pub fn trace_ray(&self, ray: &Ray, t_min: f32, t_max: f32) -> Color {
        let mut result: Option<(f32, Color)> = None;

        for object in self.objects.iter() {
            if let Some(r) = object.hit_test(ray, t_min, t_max) {
                if result.is_none() {
                    result = Some(r)
                } else if r.0 < result.unwrap().0 {
                    result = Some(r)
                }
            }
        }

        result.map_or(self.background, |r| r.1)
    }
}
