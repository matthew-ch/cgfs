use std::ops::Sub;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Vector {
        Vector {
            dx: rhs.x - self.x,
            dy: rhs.y - self.y,
            dz: rhs.z - self.z,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Vector {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl Vector {
    pub fn dot(&self, rhs: &Self) -> f32 {
        self.dx * rhs.dx + self.dy * rhs.dy + self.dz * rhs.dz
    }
}

pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}