use std::ops::{Add, Div, Index, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub type ColorPack = (u8, u8, u8);

impl Color {
    pub const fn black() -> Self {
        Color {
            r: 0.,
            g: 0.,
            b: 0.,
        }
    }

    pub const fn white() -> Self {
        Color {
            r: 255.,
            g: 255.,
            b: 255.,
        }
    }

    pub const fn red() -> Self {
        Color {
            r: 255.,
            ..Self::black()
        }
    }

    pub const fn green() -> Self {
        Color {
            g: 255.,
            ..Self::black()
        }
    }

    pub const fn blue() -> Self {
        Color {
            b: 255.,
            ..Self::black()
        }
    }

    pub fn clamp(&self) -> ColorPack {
        (
            clamp_color_component_f(self.r),
            clamp_color_component_f(self.g),
            clamp_color_component_f(self.b),
        )
    }

}

fn clamp_color_component_f(value: f32) -> u8 {
    value.round().min(255.).max(0.) as u8
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Mul<f64> for Color {

    type Output = Color;

    fn mul(self, rhs: f64) -> Color {
        Color {
            r: self.r * rhs as f32,
            g: self.g * rhs as f32,
            b: self.b * rhs as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Vector {
        Vector {
            dx: self.x - rhs.x,
            dy: self.y - rhs.y,
            dz: self.z - rhs.z,
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Point {
        Point {
            x: self.x + rhs.dx,
            y: self.y + rhs.dy,
            z: self.z + rhs.dz,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vector {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

impl Vector {
    pub fn dot(&self, rhs: &Self) -> f64 {
        self.dx * rhs.dx + self.dy * rhs.dy + self.dz * rhs.dz
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn cos(&self, rhs: &Self) -> f64 {
        let d = self.dot(rhs);
        d / (self.length() * rhs.length())
    }

    pub fn reflect(&self, rhs: &Self) -> Self {
        *self * self.dot(rhs) * 2. - *rhs
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector {
            dx: self.dx + rhs.dx,
            dy: self.dy + rhs.dy,
            dz: self.dz + rhs.dz,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector {
            dx: self.dx - rhs.dx,
            dy: self.dy - rhs.dy,
            dz: self.dz - rhs.dz,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Vector {
        Vector {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
            dz: self.dz * rhs,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Vector {
        self.mul(1. / rhs)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector {
            dx: -self.dx,
            dy: -self.dy,
            dz: -self.dz,
        }
    }
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

#[derive(Debug, Clone, Copy)]
pub struct Matrix([Vector; 3]);

impl Index<usize> for Matrix {
    type Output = Vector;

    fn index(&self, index: usize) -> &Vector {
        &self.0[index]
    }
}

impl Matrix {
    pub fn identity() -> Self {
        Matrix([
            Vector { dx: 1., dy: 0., dz: 0., },
            Vector { dx: 0., dy: 1., dz: 0. },
            Vector { dx: 0., dy: 0., dz: 1. },
        ])
    }

    pub fn rotate_x(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();

        Matrix([
            Vector { dx: 1., dy: 0., dz: 0. },
            Vector { dx: 0., dy: cos, dz: sin },
            Vector { dx: 0., dy: -sin, dz: cos },
        ])
    }

    pub fn rotate_y(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();

        Matrix([
            Vector { dx: cos, dy: 0., dz: -sin },
            Vector { dx: 0., dy: 1., dz: 0. },
            Vector { dx: sin, dy: 0., dz: cos },
        ])
    }

    pub fn rotate_z(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();

        Matrix([
            Vector { dx: cos, dy: sin, dz: 0. },
            Vector { dx: -sin, dy: cos, dz: 0. },
            Vector { dx: 0., dy: 0., dz: 1. }
        ])
    }

    pub fn compose(m: &Vec<Matrix>) -> Matrix {
        m.iter().fold(Self::identity(), |acc, &x| acc * x)
    }

    pub fn transpose(&self) -> Matrix {
        Matrix([
            Vector { dx: self[0].dx, dy: self[1].dx, dz: self[2].dx },
            Vector { dx: self[0].dy, dy: self[1].dy, dz: self[2].dy },
            Vector { dx: self[0].dz, dy: self[1].dz, dz: self[2].dz },
        ])
    }

    pub fn dot(&self, rhs: &Vector) -> Vector {
        Vector {
            dx: self[0].dot(rhs),
            dy: self[1].dot(rhs),
            dz: self[2].dot(rhs),
        }
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Matrix {
        let t = rhs.transpose();
        Matrix([
            Vector { dx: self[0].dot(&t[0]), dy: self[0].dot(&t[1]), dz: self[0].dot(&t[2]) },
            Vector { dx: self[1].dot(&t[0]), dy: self[1].dot(&t[1]), dz: self[1].dot(&t[2]) },
            Vector { dx: self[2].dot(&t[0]), dy: self[2].dot(&t[1]), dz: self[2].dot(&t[2]) },
        ])
    }
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Sphere {
    pub fn compute_ray_intersection(&self, ray: &Ray) -> Option<(f64, f64)> {
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
        Some((t1, t2))
    }
}