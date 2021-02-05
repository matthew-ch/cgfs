use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn black() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
        }
    }

    pub const fn white() -> Self {
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    pub const fn red() -> Self {
        Color {
            r: 255,
            ..Self::black()
        }
    }

    pub const fn green() -> Self {
        Color {
            g: 255,
            ..Self::black()
        }
    }

    pub const fn blue() -> Self {
        Color {
            b: 255,
            ..Self::black()
        }
    }

}

fn clamp_color_component(value: f32) -> u8 {
    value.min(255.).max(0.) as u8
}

impl Mul<f32> for Color {

    type Output = Color;

    fn mul(self, rhs: f32) -> Color {
        let r = self.r as f32 * rhs;
        let g = self.g as f32 * rhs;
        let b = self.b as f32 * rhs;
        Color {
            r: clamp_color_component(r),
            g: clamp_color_component(g),
            b: clamp_color_component(b),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl Vector {
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.dx * rhs.dx + self.dy * rhs.dy + self.dz * rhs.dz
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn cos(&self, rhs: &Self) -> f32 {
        let d = self.dot(rhs);
        d / (self.length() * rhs.length())
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

impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Vector {
        Vector {
            dx: self.dx * rhs,
            dy: self.dy * rhs,
            dz: self.dz * rhs,
        }
    }
}

impl Div<f32> for Vector {
    type Output = Vector;

    fn div(self, rhs: f32) -> Vector {
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