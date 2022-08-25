use bevy::prelude::*;
use std::{f32::consts::PI, ops::Add};

pub const INNER_RADIUS_COEFF: f32 = 0.866025404;

const SQRT_3: f32 = 1.732_f32;

/// A hex cell at a given position.
#[derive(Component, Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    // Create a new hex with the offset coordinates `q` and `r`.
    #[inline(always)]
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    #[inline]
    pub fn neighbor(self, dir: Direction, layout: &Layout) -> Self {
        let (q, r) = self.into();

        let is_even = match layout.orientation {
            Orientation::Flat => q % 2 == 0,
            Orientation::Pointy => r % 2 == 0,
        };

        match is_even {
            true => self + dir.offset_even(),
            false => self + dir.offset_odd(),
        }
    }

    #[inline]
    pub fn neighbors(self, layout: &Layout) -> [Self; 6] {
        Direction::all()
            .iter()
            .map(|d| self.neighbor(*d, layout))
            .collect::<Vec<Hex>>()
            .try_into()
            .unwrap()
    }
}

impl Add<Hex> for Hex {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl From<Hex> for (i32, i32) {
    #[inline]
    fn from(h: Hex) -> Self {
        (h.q, h.r)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

impl Direction {
    pub fn all() -> &'static [Direction; 6] {
        &[
            Direction::N,
            Direction::NE,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::NW,
        ]
    }

    pub fn offset_even(self) -> Hex {
        match self {
            Direction::N => Hex::new(1, -1),
            Direction::NE => Hex::new(1, 0),
            Direction::SE => Hex::new(0, 1),
            Direction::S => Hex::new(-1, 1),
            Direction::SW => Hex::new(-1, 0),
            Direction::NW => Hex::new(0, -1),
        }
    }

    pub fn offset_odd(self) -> Hex {
        match self {
            Direction::N => Hex::new(1, -1),
            Direction::NE => Hex::new(-1, 0),
            Direction::SE => Hex::new(0, 1),
            Direction::S => Hex::new(-1, 1),
            Direction::SW => Hex::new(1, 0),
            Direction::NW => Hex::new(0, -1),
        }
    }
}

/// Hex orientation.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Orientation {
    Flat,
    Pointy,
}

/// Transforms for a hexagonal grid to world space.
pub struct OrientationTransform {
    pub fwd_matrix: [f32; 4],
    pub inv_matrix: [f32; 4],
    pub angle: f32,
}

const POINTY: OrientationTransform = OrientationTransform {
    fwd_matrix: [SQRT_3, SQRT_3 / 2.0, 0.0, 3.0 / 2.0],
    inv_matrix: [SQRT_3 / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0],
    angle: 0.5,
};

const FLAT: OrientationTransform = OrientationTransform {
    fwd_matrix: [3.0 / 2.0, 0.0, SQRT_3 / 2.0, SQRT_3],
    inv_matrix: [2.0 / 3.0, 0.0, -1.0 / 3.0, SQRT_3 / 3.0],
    angle: 0.0,
};

impl Orientation {
    #[inline]
    pub fn transform(self) -> OrientationTransform {
        match self {
            Orientation::Flat => FLAT,
            Orientation::Pointy => POINTY,
        }
    }
}

/// A hexagonal grid layout.
#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub orientation: Orientation,
    pub origin: Vec2,
    pub size: Vec2,
}

impl Layout {
    #[inline]
    pub fn hex_to_world(&self, hex: Hex) -> Vec2 {
        let matrix = self.orientation.transform().fwd_matrix;
        let (sx, sy) = self.size.into();
        let (ox, oy) = self.origin.into();
        Vec2::new(
            matrix[0].mul_add(hex.q as f32, matrix[1] * hex.r as f32) * sx + ox,
            matrix[2].mul_add(hex.q as f32, matrix[3] * hex.r as f32) * sy + oy,
        )
    }

    #[inline]
    pub fn world_to_hex(&self, pos: Vec2) -> Hex {
        let matrix = self.orientation.transform().inv_matrix;
        let point = (pos - self.origin) / self.size;
        let x = matrix[0].mul_add(point.x, matrix[1] * point.y);
        let y = matrix[2].mul_add(point.x, matrix[3] * point.y);
        Hex::new(x.round() as i32, y.round() as i32)
    }

    #[inline]
    pub fn hex_corners(&self, hex: Hex) -> [Vec2; 6] {
        let center = self.hex_to_world(hex);
        [0, 1, 2, 3, 4, 5].map(|corner| {
            let angle = PI * 2.0 * (self.orientation.transform().angle + corner as f32) / 6.;
            center + Vec2::new(self.size.x * angle.cos(), self.size.y * angle.sin())
        })
    }

    #[inline]
    pub fn hex_world_size(&self) -> (f32, f32) {
        let (sx, sy) = self.size.into();
        let sx = match self.orientation {
            Orientation::Flat => sx,
            Orientation::Pointy => sx * INNER_RADIUS_COEFF,
        };

        let sy = match self.orientation {
            Orientation::Flat => sy * INNER_RADIUS_COEFF,
            Orientation::Pointy => sy,
        };
        (sx, sy)
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            orientation: Orientation::Pointy,
            origin: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
        }
    }
}

/// Generates a rectangle layout with given width `w` and height `h` on given orientation `o`.
pub fn rectangle(w: i32, h: i32, o: Orientation) -> impl Iterator<Item = Hex> {
    match o {
        Orientation::Flat => rectangle_flat(w, h),
        Orientation::Pointy => rectangle_pointy(w, h),
    }
}

fn rectangle_pointy(w: i32, h: i32) -> Box<dyn Iterator<Item = Hex>> {
    Box::new((0..=h).flat_map(move |y| (0 - (y >> 1)..w - (y >> 1)).map(move |x| Hex::new(x, y))))
}

fn rectangle_flat(w: i32, h: i32) -> Box<dyn Iterator<Item = Hex>> {
    Box::new((0..=w).flat_map(move |x| (0 - (x >> 1)..h - (x >> 1)).map(move |y| Hex::new(x, y))))
}
