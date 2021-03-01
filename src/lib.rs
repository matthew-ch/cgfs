mod components;
mod objects;
pub use components::*;
pub use objects::*;
use std::thread;
use std::mem;
use std::ops::RangeInclusive;

pub(crate) const EPS: f64 = 0.001;
pub(crate) const AIR_REFRACTION_INDEX: f64 = 1.0;

fn interpolate(i0: i32, d0: f64, i1: i32, d1: f64) -> Vec<f64> {
    if i0 == i1 {
        vec![d0]
    } else {
        let a = (d1 - d0) / (i1 - i0) as f64;
        (i0..=i1).into_iter().map(|i| (i - i0) as f64 * a + d0).collect()
    }
}

pub struct Canvas {
    width: u32,
    height: u32,
    image_data: Vec<ColorPack>,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background: Color) -> Canvas {
        assert!(width > 0 && height > 0);

        let image_data = vec![background.clamp(); width as usize * height as usize];

        Canvas {
            width,
            height,
            image_data,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width && y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let index = self.index(x, y);
        self.image_data[index] = color.clamp();
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        let x = x + self.width as i32 / 2;
        let y = self.height as i32 / 2 - y;
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return;
        }
        self.set_pixel(x as u32, y as u32, color);
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

    pub fn render(&mut self, scene: &Scene, depth: u32, samples: u32) {
        let width = self.width;
        let height = self.height;
        for x in 0..width {
            for y in 0..height {
                self.super_sampling(x, y, samples, scene, depth);
            }
        }
    }

    pub fn render_mth(&mut self, scene: &Scene, threads: u32, depth: u32, samples: u32) {
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

    fn super_sampling(&mut self, x: u32, y: u32, n: u32, scene: &Scene, depth: u32) {
        let mut color = Color::black();
        let width = self.width * n;
        let height = self.height * n;
        let half_width = (width / 2) as i32;
        let half_height = (height / 2) as i32;
        let xn = (x * n) as i32;
        let yn = (y * n) as i32;
        for i in 0..n as i32 {
            for j in 0..n as i32 {
                let ray = scene.canvas_to_viewport(xn + i - half_width, half_height - (yn + j), width, height);
                color = color + scene.trace_ray(&ray, AIR_REFRACTION_INDEX, 1.0..=f64::INFINITY, depth);
            }
        }
        self.set_pixel(x, y, color * (1. / (n * n) as f64));
    }

    pub fn draw_line(&mut self, mut p0: Point, mut p1: Point, color: Color) {
        let x_ys: Box<dyn Iterator<Item = (i32, i32)>> = if (p0.x - p1.x).abs() > (p0.y - p1.y).abs() {
            if p0.x > p1.x {
                mem::swap(&mut p0, &mut p1);
            }
            let x0 = p0.x.round() as i32;
            let x1 = p1.x.round() as i32;
            Box::new(
                (x0..=x1).into_iter()
                    .zip(interpolate(x0, p0.y, x1, p1.y).into_iter().map(|y| y as i32))
            )
        } else {
            if p0.y > p1.y {
                mem::swap(&mut p0, &mut p1);
            }
            let y0 = p0.y.round() as i32;
            let y1 = p1.y.round() as i32;
            Box::new(
                interpolate(y0, p0.x, y1, p1.x).into_iter().map(|x| x as i32)
                    .zip((y0..=y1).into_iter())
            )
        };
        for (x, y) in x_ys {
            self.put_pixel(x, y, color);
        }
    }

    pub fn draw_wireframe_triangle(&mut self, p0: Point, p1: Point, p2: Point, color: Color) {
        self.draw_line(p0, p1, color);
        self.draw_line(p1, p2, color);
        self.draw_line(p2, p0, color);
    }

    pub fn draw_shaded_triangle(&mut self, mut p0: Point, mut p1: Point, mut p2: Point, color: Color) {
        if p1.y < p0.y {
            mem::swap(&mut p1, &mut p0);
        }
        if p2.y < p0.y {
            mem::swap(&mut p2, &mut p0);
        }
        if p2.y < p1.y {
            mem::swap(&mut p2, &mut p1);
        }
        let y0 = p0.y.round() as i32;
        let y1 = p1.y.round() as i32;
        let y2 = p2.y.round() as i32;
        let x02 = interpolate(y0, p0.x, y2, p2.x);
        let h02 = interpolate(y0, p0.z, y2, p2.z);
        let x012 = {
            let mut x01 = interpolate(y0, p0.x, y1, p1.x);
            let mut x12 = interpolate(y1, p1.x, y2, p2.x);
            x01.pop();
            x01.append(&mut x12);
            x01
        };
        let h012 = {
            let mut h01 = interpolate(y0, p0.z, y1, p1.z);
            let mut h12 = interpolate(y1, p1.z, y2, p2.z);
            h01.pop();
            h01.append(&mut h12);
            h01
        };
        assert!(x02.len() == x012.len());
        let (x_left, x_right, h_left, h_right) = {
            let m = x02.len() / 2;
            if x02[m] < x012[m] {
                (&x02, &x012, &h02, &h012)
            } else {
                (&x012, &x02, &h012, &h02)
            }
        };
        for y in y0..=y2 {
            let i = (y - y0) as usize;
            let l = x_left[i].round() as i32;
            let r = x_right[i].round() as i32;
            let hs = interpolate(l, h_left[i], r, h_right[i]);
            for x in l..=r {
                self.put_pixel(x, y, color * hs[(x - l) as usize]);
            }
        }
    }

    fn render_triangle(&mut self, triangle: &([usize; 3], Color), projected: &Vec<Point>) {
        self.draw_wireframe_triangle(
            projected[triangle.0[0]], 
            projected[triangle.0[1]], 
            projected[triangle.0[2]],
            triangle.1,
        );
    }

    pub fn rasterize(&mut self, scene: &Scene) {
        for instance in scene.instances.iter() {
            let model = scene.models.iter().find(|&model| model.name == instance.model_name);
            if model.is_none() {
                continue;
            }
            let model = model.unwrap();
            let projected: Vec<Point> = model.vertices.iter()
                .map(|&v| Point::from(v) + instance.position)
                .map(|v| scene.viewport_to_canvas(v, self.width, self.height))
                .collect();
            for t in model.triangles.iter() {
                self.render_triangle(t, &projected);
            }
        }
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
    models: Vec<SceneModel>,
    instances: Vec<SceneModelInstance>,
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
            models: Vec::new(),
            instances: Vec::new(),
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

    pub fn add_model(&mut self, model: SceneModel) {
        self.models.push(model);
    }

    pub fn add_instance(&mut self, instance: SceneModelInstance) {
        self.instances.push(instance);
    }

    pub fn canvas_to_viewport(&self, x: i32, y: i32, width: u32, height: u32) -> Ray {
        Ray {
            origin: self.camera_position,
            direction: self.camera_rotation.dot(&Vector {
                dx: x as f64 / width as f64 * self.viewport_width,
                dy: y as f64 / height as f64 * self.viewport_height,
                dz: self.camera_distance,
            })
        }
    }

    pub fn viewport_to_canvas(&self, point: Point, width: u32, height: u32) -> Point {
        let factor = self.camera_distance / point.z;
        Point {
            x: point.x * width as f64 / self.viewport_width * factor,
            y: point.y * height as f64 / self.viewport_height * factor,
            z: factor,
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
