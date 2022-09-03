use std::{f32::consts::PI, ops::Add};

use bevy::prelude::*;

pub const INNER_RADIUS_COEFF: f32 = 0.866025404;

const SQRT_3: f32 = 1.732_f32;

/// A hex in axial-coordinates.
#[derive(Component, Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Coord {
    pub q: i32,
    pub r: i32,
}

impl Coord {
    /// Create a hex axial-coordinate on `q` and `r` axis.
    #[inline]
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn neighbor(self, dir: Direction) -> Self {
        self + dir.offset()
    }

    pub fn neighbors(&self) -> [Self; 6] {
        Direction::all()
            .iter()
            .map(|d| self.neighbor(*d))
            .collect::<Vec<Coord>>()
            .try_into()
            .unwrap()
    }
}

impl Add<Coord> for Coord {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl From<Coord> for (i32, i32) {
    #[inline]
    fn from(h: Coord) -> Self {
        (h.q, h.r)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Bounds {
    pub mins: Vec2,
    pub maxs: Vec2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    /// (1, 0)
    A,
    /// (1, -1)
    B,
    /// (0, -1)
    C,
    /// (-1, 0)
    D,
    /// (-1, 1)
    E,
    /// (0, 1)
    F,
}

impl Direction {
    /// ```txt
    ///            x Axis
    ///            ___
    ///           /   \
    ///       +--+  2  +--+
    ///      / 3  \___/  1 \
    ///      \    /   \    /
    ///       +--+     +--+
    ///      /    \___/    \
    ///      \ 4  /   \  0 /
    ///       +--+  5  +--+   y Axis
    ///           \___/
    /// ```
    pub fn all() -> &'static [Direction; 6] {
        &[
            Direction::A,
            Direction::B,
            Direction::C,
            Direction::D,
            Direction::E,
            Direction::F,
        ]
    }
    pub fn offset(&self) -> Coord {
        match self {
            Direction::A => Coord::new(1, 0),
            Direction::B => Coord::new(1, -1),
            Direction::C => Coord::new(0, -1),
            Direction::D => Coord::new(-1, 0),
            Direction::E => Coord::new(-1, 1),
            Direction::F => Coord::new(0, 1),
        }
    }
}

/// Hexagon orientation coefficients. Often times either [Orientation.pointy] or [Orientation.flat] orientation is used.
#[derive(Debug, Clone)]
pub struct Orientation {
    fwd_matrix: [f32; 4],
    inv_matrix: [f32; 4],
    angle: f32,
}

const POINTY: Orientation = Orientation {
    fwd_matrix: [SQRT_3, SQRT_3 / 2.0, 0.0, 3.0 / 2.0],
    inv_matrix: [SQRT_3 / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0],
    angle: 0.5,
};

const FLAT: Orientation = Orientation {
    fwd_matrix: [3.0 / 2.0, 0.0, SQRT_3 / 2.0, SQRT_3],
    inv_matrix: [2.0 / 3.0, 0.0, -1.0 / 3.0, SQRT_3 / 3.0],
    angle: 0.0,
};

impl Orientation {
    pub fn pointy() -> &'static Self {
        &POINTY
    }

    pub fn flat() -> &'static Self {
        &FLAT
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub orientation: Orientation,
    pub size: Vec2,
    pub origin: Vec2,
}

impl Layout {
    pub fn new(orientation: &Orientation, size: Vec2, origin: Vec2) -> Self {
        Layout {
            orientation: orientation.clone(),
            size,
            origin,
        }
    }

    pub fn is_pointy(&self) -> bool {
        (f32::from(self.orientation.angle) - 0.5).abs() <= 0.1
    }

    pub fn is_flat(&self) -> bool {
        f32::from(self.orientation.angle).abs() <= 0.1
    }

    /// Create a hex axial-coordinate from world `pos`.
    pub fn from_world(&self, pos: Vec3) -> Coord {
        let pos_2d = Vec2::new(pos.x, pos.z);
        let matrix = self.orientation.inv_matrix;
        let point = (pos_2d - self.origin) / self.size;
        let x = matrix[0].mul_add(point.x, matrix[1] * point.y);
        let y = matrix[2].mul_add(point.x, matrix[3] * point.y);
        Coord::new(x.round() as i32, y.round() as i32)
    }

    /// Convert a hex axial-coordinate to world position.
    pub fn to_world(&self, hex: Coord) -> Vec2 {
        let matrix = self.orientation.fwd_matrix;
        let (sx, sy) = self.size.into();
        let (ox, oy) = self.origin.into();
        Vec2::new(
            matrix[0].mul_add(hex.q as f32, matrix[1] * hex.r as f32) * sx + ox,
            matrix[2].mul_add(hex.q as f32, matrix[3] * hex.r as f32) * sy + oy,
        )
    }

    /// Convert a hex axial-coordinate to world position with given `y` value.
    pub fn to_world_y(&self, hex: Coord, y: f32) -> Vec3 {
        let pos = self.to_world(hex);
        Vec3::new(pos.x, y, pos.y)
    }

    /// Returns the world position of the hex corners.
    pub fn hex_corners(&self, hex: Coord) -> [Vec2; 6] {
        let center = self.to_world(hex);
        [0, 1, 2, 3, 4, 5].map(|corner| {
            let angle = PI * 2.0 * (self.orientation.angle + corner as f32) / 6.;
            center + Vec2::new(self.size.x * angle.cos(), self.size.y * angle.sin())
        })
    }

    /// Returns the rectangal bounding box of a hex.
    pub fn hex_rect_bounds(&self, hex: Coord) -> Bounds {
        let mut x_min = f32::NAN;
        let mut x_max = f32::NAN;
        let mut y_min = f32::NAN;
        let mut y_max = f32::NAN;
        for point in self.hex_corners(hex).iter() {
            x_min = x_min.min(point.x);
            x_max = x_max.max(point.x);
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
        }
        Bounds {
            mins: Vec2::new(x_min, y_min),
            maxs: Vec2::new(x_max, y_max),
        }
    }

    /// Returns the hex width and height.
    pub fn hex_size(&self) -> (f32, f32) {
        let (sx, sy) = self.size.into();
        if self.is_pointy() {
            (sx * INNER_RADIUS_COEFF, sy)
        } else {
            (sx, sy * INNER_RADIUS_COEFF)
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            orientation: Orientation::pointy().clone(),
            origin: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
        }
    }
}

/// Generates a rectangle odd-r shape with given width `w` and height `h` on given layout `layout`.
pub fn rectangle(w: i32, h: i32, layout: &Layout) -> impl Iterator<Item = Coord> {
    match layout.is_pointy() {
        true => rectangle_pointy(w, h),
        false => rectangle_flat(w, h),
    }
}

fn rectangle_pointy(w: i32, h: i32) -> Box<dyn Iterator<Item = Coord>> {
    Box::new((0..=h).flat_map(move |y| (0 - (y >> 1)..w - (y >> 1)).map(move |x| Coord::new(x, y))))
}

fn rectangle_flat(w: i32, h: i32) -> Box<dyn Iterator<Item = Coord>> {
    Box::new((0..=w).flat_map(move |x| (0 - (x >> 1)..h - (x >> 1)).map(move |y| Coord::new(x, y))))
}
