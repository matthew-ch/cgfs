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