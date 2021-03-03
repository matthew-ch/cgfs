use std::ops::{Add, Div, Mul, Neg, Sub};

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

    pub const fn yellow() -> Self {
        Color {
            r: 255.,
            g: 255.,
            b: 0.,
        }
    }

    pub const fn purple() -> Self {
        Color {
            r: 255.,
            g: 0.,
            b: 255.,
        }
    }

    pub const fn cyan() -> Self {
        Color {
            r: 0.,
            g: 255.,
            b: 255.,
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

#[derive(Debug, Clone, Copy)]
pub struct HomogeneousCoordinate(f64, f64, f64, f64);

impl HomogeneousCoordinate {

    pub fn x(&self) -> f64 {
        self.0
    }

    pub fn y(&self) -> f64 {
        self.1
    }

    pub fn z(&self) -> f64 {
        self.2
    }

    pub fn w(&self) -> f64 {
        self.3
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2 + self.3 * rhs.3
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self(
            self.1 * rhs.2 - self.2 * rhs.1,
            self.2 * rhs.0 - self.0 * rhs.2,
            self.0 * rhs.1 - self.1 * rhs.0,
            0.,
        )
    }

    pub fn cos(&self, rhs: &Self) -> f64 {
        let d = self.dot(rhs);
        d / (self.length() * rhs.length())
    }

    pub fn reflect(&self, rhs: &Self) -> Self {
        *self * self.dot(rhs) * 2. - *rhs
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn canonical(&self) -> Self {
        if self.3 == 0. || self.3 == 1. {
            *self
        } else {
            Self(self.0 / self.3, self.1 / self.3, self.2 / self.3, 1.)
        }
    }

}

impl Add for HomogeneousCoordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}

impl Sub for HomogeneousCoordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl Mul<f64> for HomogeneousCoordinate {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Self(
            self.0 * rhs,
            self.1 * rhs,
            self.2 * rhs,
            self.3 * rhs,
        )
    }
}

impl Div<f64> for HomogeneousCoordinate {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        self * (1. / rhs)
    }
}

impl Neg for HomogeneousCoordinate {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0, -self.1, -self.2, -self.3)
    }
}

impl From<[f64; 4]> for HomogeneousCoordinate {
    fn from(value: [f64; 4]) -> Self {
        Self(value[0], value[1], value[2], value[3])
    }
}

impl From<[f64; 3]> for HomogeneousCoordinate {
    fn from(value: [f64; 3]) -> Self {
        Self(value[0], value[1], value[2], 1.)
    }
}

impl From<(f64, f64, f64, f64)> for HomogeneousCoordinate {
    fn from(value: (f64, f64, f64, f64)) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

impl From<(f64, f64, f64)> for HomogeneousCoordinate {
    fn from(value: (f64, f64, f64)) -> Self {
        Self(value.0, value.1, value.2, 1.)
    }
}

pub type Point = HomogeneousCoordinate;
pub type Vector = HomogeneousCoordinate;

#[derive(Debug, Clone, Copy)]
pub struct HomogeneousMatrix([[f64; 4];4]);

impl HomogeneousMatrix {
    pub fn new(mat: [[f64; 4]; 4]) -> Self {
        Self(mat)
    }

    pub fn identity() -> Self {
        Self([
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn scale(s: f64) -> Self {
        Self([
            [s, 0., 0., 0.],
            [0., s, 0., 0.],
            [0., 0., s, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn translation(tx: f64, ty: f64, tz: f64) -> Self {
        Self([
            [1., 0., 0., tx],
            [0., 1., 0., ty],
            [0., 0., 1., tz],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotation_x(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        Self([
            [1., 0., 0., 0.],
            [0., cos, -sin, 0.],
            [0., sin, cos, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotation_y(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        Self([
            [cos, 0., sin, 0.],
            [0., 1., 0., 0.],
            [-sin, 0., cos, 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn rotation_z(deg: f64) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        Self([
            [cos, -sin, 0., 0.],
            [sin, cos, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ])
    }

    pub fn compose(m: Vec<Self>) -> Self {
        m.into_iter().fold(Self::identity(), |acc, x| acc * x)
    }

    pub fn dot(&self, rhs: &HomogeneousCoordinate) -> HomogeneousCoordinate {
        HomogeneousCoordinate(
            HomogeneousCoordinate::from(self.0[0]).dot(rhs),
            HomogeneousCoordinate::from(self.0[1]).dot(rhs),
            HomogeneousCoordinate::from(self.0[2]).dot(rhs),
            HomogeneousCoordinate::from(self.0[3]).dot(rhs),
        )
    }

    pub fn transposed(&self) -> Self {
        let mat = &self.0;
        Self([
            [mat[0][0], mat[1][0], mat[2][0], mat[3][0]],
            [mat[0][1], mat[1][1], mat[2][1], mat[3][1]],
            [mat[0][2], mat[1][2], mat[2][2], mat[3][2]],
            [mat[0][3], mat[1][3], mat[2][3], mat[3][3]],
        ])
    }
}

impl Mul for HomogeneousMatrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let l_mat = &self.0;
        let r_mat = &rhs.0;
        let mut mat = [[0.; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    mat[i][j] += l_mat[i][k] * r_mat[k][j];
                }
            }
        }
        Self(mat)
    }
}

pub type Matrix = HomogeneousMatrix;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub specular: i32,
    pub reflective: f64,
    pub transparency: Option<f64>,
}

pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
    pub normal: Vector,
}

fn solve_equations(mut coefficients: [[f64; 3]; 3], mut rhs: [f64; 3]) -> Option<[f64; 3]> {
    for i in 0..3 {
        if coefficients[i][i] == 0. {
            for j in (i+1)..3 {
                if coefficients[j][i] != 0. {
                    let temp = coefficients[i];
                    coefficients[i] = coefficients[j];
                    coefficients[j] = temp;
                    let temp = rhs[i];
                    rhs[i] = rhs[j];
                    rhs[j] = temp;
                }
                break;
            }
        }
        if coefficients[i][i] == 0. {
            return None
        }
        let c = coefficients[i][i];
        for j in i..3 {
            coefficients[i][j] /= c;
        }
        rhs[i] /= c;
        for j in 0..3 {
            if j == i {
                continue;
            }
            let c = coefficients[j][i];
            for k in i..3 {
                coefficients[j][k] -= c * coefficients[i][k];
            }
            rhs[j] -= c * rhs[i];
        }
    }
    for i in 0..3 {
        if coefficients[i][i] != 1. {
            return None
        }
    }
    Some(rhs)
}

impl Triangle {
    pub fn new(a: Point, b: Point, c: Point) -> Triangle {
        let v: Vector = b - a;
        let w: Vector = c - a;
        let cross = v.cross(&w);
        Triangle {
            a,
            b,
            c,
            normal: cross / cross.length(),
        }
    }

    pub fn compute_ray_intersection(&self, ray: &Ray) -> Option<f64> {
        let ab: Vector = self.b - self.a;
        let ac: Vector = self.c - self.a;
        let ao: Vector = ray.origin - self.a;
        solve_equations([
            [ab.x(), ac.x(), -ray.direction.x()],
            [ab.y(), ac.y(), -ray.direction.y()],
            [ab.z(), ac.z(), -ray.direction.z()],
        ], [ao.x(), ao.y(), ao.z()])
        .map(|[r, s, t]| if r < 0. || s < 0. || r + s > 1. { None } else { Some(t) })
        .flatten()
    }
}

pub struct Plane {
    pub normal: Vector,
    pub d: f64,
}

impl Plane {
    pub fn from_points(a: Point, b: Point, c: Point) -> Self {
        let v: Vector = b - a;
        let w: Vector = c - a;
        let cross = v.cross(&w);
        let normal: Vector = cross / cross.length();
        let d = -normal.dot(&a);
        Self {
            normal,
            d,
        }
    }

    pub fn signed_distance(&self, p: &Point) -> f64 {
        self.normal.dot(p) + self.d
    }

    pub fn intersection(&self, a: &Point, b: &Point) -> Option<(f64, Point)> {
        let ab = *b - *a;
        let denom = self.normal.dot(&ab);
        if denom == 0. {
            None
        } else {
            let t = (-self.d - self.normal.dot(a)) / denom;
            Some((t, *a + ab * t))
        }
    }
}