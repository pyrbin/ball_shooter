use bevy::prelude::*;
use std::collections::HashMap;

/// A hex cell at a given position.
#[derive(Component, Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl Hex {
    // Create a new hex with the offset coordinates `q` and `r`.
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r, s: -q - r }
    }
}

const SQRT_3: f32 = 1.732_f32;

/// Hex orientation.
#[derive(Debug, Copy, Clone)]
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
    pub fn hex_to_world(&self, hex: Hex) -> Vec2 {
        let matrix = self.orientation.transform().fwd_matrix;
        let (sx, sy) = self.size.into();
        let (ox, oy) = self.origin.into();
        Vec2::new(
            matrix[0].mul_add(hex.q as f32, matrix[1] * hex.r as f32) * sx + ox,
            matrix[2].mul_add(hex.q as f32, matrix[3] * hex.r as f32) * sy + oy,
        )
    }

    pub fn world_to_hex(&self, pos: Vec2) -> Hex {
        let matrix = self.orientation.transform().inv_matrix;
        let point = (pos - self.origin) / self.size;
        let x = matrix[0].mul_add(point.x, matrix[1] * point.y);
        let y = matrix[2].mul_add(point.x, matrix[3] * point.y);
        Hex::new(x.round() as i32, y.round() as i32)
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

/// A hexagonal board.
#[derive(Default, Debug, Clone)]
pub struct Board {
    pub width: u32,
    pub height: u32,
    pub layout: Layout,
    pub storage: HashMap<Hex, Entity>,
}

impl Board {
    pub fn get(&self, hex: Hex) -> Option<&Entity> {
        self.storage.get(&hex)
    }

    pub fn set(&mut self, hex: Hex, entity: Option<Entity>) -> Option<Entity> {
        match entity {
            Some(entity) => self.storage.insert(hex.clone(), entity),
            None => self.storage.remove(&hex),
        }
    }

    pub fn hex_to_world(&self, hex: Hex) -> Vec2 {
        self.layout.hex_to_world(hex)
    }

    pub fn hex_to_world_y(&self, hex: Hex, y: f32) -> Vec3 {
        let pos_2d = self.layout.hex_to_world(hex);
        Vec3::new(pos_2d.x, y, pos_2d.y)
    }

    pub fn world_to_hex(&self, pos: Vec2) -> Hex {
        self.layout.world_to_hex(pos)
    }
}

/// Generates a rectangle layout for given `board`. Doesn't insert any cells into the board.
pub fn rectangle(b: &Board) -> impl Iterator<Item = Hex> {
    match b.layout.orientation {
        Orientation::Flat => rectangle_flat(b),
        Orientation::Pointy => rectangle_pointy(b),
    }
}

fn rectangle_pointy(b: &Board) -> Box<dyn Iterator<Item = Hex>> {
    let (w, h) = (b.width as i32, b.height as i32);
    Box::new((0..=h).flat_map(move |y| (0 - (y >> 1)..w - (y >> 1)).map(move |x| Hex::new(x, y))))
}

fn rectangle_flat(b: &Board) -> Box<dyn Iterator<Item = Hex>> {
    let (w, h) = (b.width as i32, b.height as i32);
    Box::new((0..=w).flat_map(move |x| (0 - (x >> 1)..h - (x >> 1)).map(move |y| Hex::new(x, y))))
}
