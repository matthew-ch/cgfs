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
    depth_buffer: Vec<f64>,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background: Color) -> Canvas {
        assert!(width > 0 && height > 0);

        let image_data = vec![background.clamp(); (width * height) as usize];

        Canvas {
            width,
            height,
            image_data,
            depth_buffer: vec![0.; (width * height) as usize],
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

    fn update_depth_buffer(&mut self, x: i32, y: i32, iz: f64) -> bool {
        let x = x + self.width as i32 / 2;
        let y = self.height as i32 / 2 - y;
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            return false;
        }
        let index = self.index(x as u32, y as u32);
        if self.depth_buffer[index] < iz {
            self.depth_buffer[index] = iz;
            true
        } else {
            false
        }
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
        let x_ys: Box<dyn Iterator<Item = (i32, i32)>> = if (p0.x() - p1.x()).abs() > (p0.y() - p1.y()).abs() {
            if p0.x() > p1.x() {
                mem::swap(&mut p0, &mut p1);
            }
            let x0 = p0.x().round() as i32;
            let x1 = p1.x().round() as i32;
            Box::new(
                (x0..=x1).into_iter()
                    .zip(interpolate(x0, p0.y(), x1, p1.y()).into_iter().map(|y| y as i32))
            )
        } else {
            if p0.y() > p1.y() {
                mem::swap(&mut p0, &mut p1);
            }
            let y0 = p0.y().round() as i32;
            let y1 = p1.y().round() as i32;
            Box::new(
                interpolate(y0, p0.x(), y1, p1.x()).into_iter().map(|x| x as i32)
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
        if p1.y() < p0.y() {
            mem::swap(&mut p1, &mut p0);
        }
        if p2.y() < p0.y() {
            mem::swap(&mut p2, &mut p0);
        }
        if p2.y() < p1.y() {
            mem::swap(&mut p2, &mut p1);
        }
        let y0 = p0.y().round() as i32;
        let y1 = p1.y().round() as i32;
        let y2 = p2.y().round() as i32;
        let x02 = interpolate(y0, p0.x(), y2, p2.x());
        let iz02 = interpolate(y0, 1. / p0.z(), y2, 1. / p2.z());
        let x012 = {
            let mut x01 = interpolate(y0, p0.x(), y1, p1.x());
            let mut x12 = interpolate(y1, p1.x(), y2, p2.x());
            x01.pop();
            x01.append(&mut x12);
            x01
        };
        let iz012 = {
            let mut iz01 = interpolate(y0, 1. / p0.z(), y1, 1. / p1.z());
            let mut iz12 = interpolate(y1, 1. / p1.z(), y2, 1. / p2.z());
            iz01.pop();
            iz01.append(&mut iz12);
            iz01
        };
        assert!(x02.len() == x012.len());
        let (x_left, x_right, iz_left, iz_right) = {
            let m = x02.len() / 2;
            if x02[m] < x012[m] {
                (&x02, &x012, &iz02, &iz012)
            } else {
                (&x012, &x02, &iz012, &iz02)
            }
        };
        for y in y0..=y2 {
            let i = (y - y0) as usize;
            let l = x_left[i].round() as i32;
            let r = x_right[i].round() as i32;
            let izs = interpolate(l, iz_left[i], r, iz_right[i]);
            for x in l..=r {
                let iz = izs[(x - l) as usize];
                if self.update_depth_buffer(x, y, iz) {
                    self.put_pixel(x, y, color);
                }
            }
        }
    }

    fn render_triangle(&mut self, triangle: &([usize; 3], Color), projected: &Vec<Point>, model_vertices: &Vec<Point>) {
        let [i, j, k] = triangle.0;
        let normal = Triangle::new(model_vertices[i], model_vertices[j], model_vertices[k]).normal;
        let center = (model_vertices[i] + model_vertices[j] + model_vertices[k]) / 3.;
        if normal.dot(&center) < 0. {
            self.draw_shaded_triangle(
                projected[i], 
                projected[j], 
                projected[k],
                triangle.1,
            );
            // self.draw_wireframe_triangle(projected[i], projected[j], projected[k], triangle.1 * 0.7);
        }
    }

    pub fn rasterize(&mut self, scene: &Scene) {
        let projection = scene.get_projection_matrix(self.width, self.height);
        let camera = scene.get_camera_matrix();

        let clipping_planes = scene.get_clipping_planes();
        for instance in scene.instances.iter() {
            let model = scene.models.iter().find(|&model| model.name == instance.model_name).expect("no model found for instance");
            let transform: Matrix = camera * instance.transform;
            let vertices: Vec<Point> = model.vertices.iter()
                .map(|&v| transform.dot(&v.into()))
                .collect();
            let model = SceneModel::new(model.name.clone(), vertices, model.triangles.clone());
            if let Some(model) = Self::clip_model(&clipping_planes, model) {
                let vertices = model.vertices.iter()
                    .map(|v| {
                        let mut p = projection.dot(v).canonical();
                        p.set_z(v.z());
                        p
                    })
                    .collect::<Vec<_>>();
                for t in model.triangles.iter() {
                    self.render_triangle(t, &vertices, &model.vertices);
                }
            }
        }
    }

    fn clip_model(clipping_planes: &Vec<Plane>, mut model: SceneModel) -> Option<SceneModel> {
        let bounding_sphere = model.get_bounding_sphere();
        let mut intersection_planes = Vec::new();
        for plane in clipping_planes {
            let d = plane.signed_distance(&bounding_sphere.center);
            if d < -bounding_sphere.radius {
                return None;
            }
            if d < bounding_sphere.radius {
                intersection_planes.push(plane);
            }
        }
        if intersection_planes.len() == 0 {
            return Some(model);
        }
        let SceneModel { mut vertices, mut triangles, name, ..} = model;
        for plane in intersection_planes {
            let trs = triangles;
            triangles = Vec::new();
            for (vids, color) in trs {
                let mut distance_id_pairs = vids.iter()
                    .map(|&vid| (plane.signed_distance(&vertices[vid]), vid))
                    .collect::<Vec<_>>();

                loop {
                    if distance_id_pairs[0].0 < distance_id_pairs[1].0 || distance_id_pairs[0].0 < distance_id_pairs[2].0 {
                        distance_id_pairs.rotate_left(1);
                    } else {
                        break;
                    }
                }

                if distance_id_pairs[0].0 <= 0. {
                    continue
                } else if distance_id_pairs[1].0 >= 0. && distance_id_pairs[2].0 >= 0. {
                    triangles.push((vids, color));
                } else if distance_id_pairs[1].0 <= 0. && distance_id_pairs[2].0 <= 0. {
                    let (_tb, b) = plane.intersection(&vertices[distance_id_pairs[0].1], &vertices[distance_id_pairs[1].1]).unwrap();
                    let (_tc, c) = plane.intersection(&vertices[distance_id_pairs[0].1], &vertices[distance_id_pairs[2].1]).unwrap();
                    let l = vertices.len();
                    vertices.push(b);
                    vertices.push(c);
                    triangles.push(([distance_id_pairs[0].1, l, l + 1], color));
                } else {
                    if distance_id_pairs[2].0 > 0. {
                        distance_id_pairs.rotate_right(1);
                    }
                    let (_ta, a) = plane.intersection(&vertices[distance_id_pairs[0].1], &vertices[distance_id_pairs[2].1]).unwrap();
                    let (_tb, b) = plane.intersection(&vertices[distance_id_pairs[1].1], &vertices[distance_id_pairs[2].1]).unwrap();
                    let l = vertices.len();
                    vertices.push(a);
                    vertices.push(b);
                    triangles.push(([distance_id_pairs[0].1, distance_id_pairs[1].1, l], color));
                    triangles.push(([distance_id_pairs[1].1, l + 1, l], color));
                }
            }
        }
        if triangles.len() == 0 {
            None
        } else {
            Some(SceneModel::new(
                name,
                vertices,
                triangles,
            ))
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
            camera_position: (0., 0., 0.).into(),
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
            direction: self.camera_rotation.dot(&(
                x as f64 / width as f64 * self.viewport_width,
                y as f64 / height as f64 * self.viewport_height,
                self.camera_distance,
                0.,
            ).into())
        }
    }

    pub fn get_camera_matrix(&self) -> Matrix {
        let pos = self.camera_position;
        Matrix::compose(vec![
            self.camera_rotation.transposed(),
            Matrix::translation(-pos.x(), -pos.y(), -pos.z()),
        ])
    }

    pub fn get_projection_matrix(&self, canvas_width: u32, canvas_height: u32) -> Matrix {
        Matrix::compose(vec![
            {
                let d = self.camera_distance;
                let mut mat = [[0.; 4]; 4];
                mat[0][0] = d;
                mat[1][1] = d;
                mat[2][2] = 1.;
                mat[3][2] = 1.;
                Matrix::new(mat)
            },
            {
                let mut mat = [[0.; 4]; 4];
                mat[0][0] = canvas_width as f64 / self.viewport_width;
                mat[1][1] = canvas_height as f64 / self.viewport_height;
                mat[2][2] = 1.;
                mat[3][3] = 1.;
                Matrix::new(mat)
            }
        ])
    }

    pub fn get_clipping_planes(&self) -> Vec<Plane> {
        let camera_pos = Point::from((0., 0., 0.));
        let top_left = Point::from((-self.viewport_width / 2., self.viewport_height / 2., self.camera_distance));
        let top_right = Point::from((self.viewport_width / 2., self.viewport_height / 2., self.camera_distance));
        let bottom_right = Point::from((self.viewport_width / 2., -self.viewport_height / 2., self.camera_distance));
        let bottom_left = Point::from((-self.viewport_width / 2., -self.viewport_height / 2., self.camera_distance));
        vec![
            Plane { normal: (0., 0., 1., 0.).into(), d: -self.camera_distance },
            Plane::from_points(camera_pos, top_left, bottom_left),
            Plane::from_points(camera_pos, top_right, top_left),
            Plane::from_points(camera_pos, bottom_right, top_right),
            Plane::from_points(camera_pos, bottom_left, bottom_right),
        ]
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
