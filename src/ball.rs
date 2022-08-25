use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::hex;

pub const BALL_RADIUS_COEFF: f32 = hex::INNER_RADIUS_COEFF * 0.85;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub enum Species {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

pub fn species_to_color(species: Species) -> Color {
    match species {
        Species::Red => Color::rgb_u8(244, 47, 47),
        Species::Blue => Color::rgb_u8(0, 93, 234),
        Species::Green => Color::rgb_u8(0, 197, 171),
        Species::Yellow => Color::rgb_u8(255, 219, 0),
        Species::Purple => Color::rgb_u8(122, 1, 218),
    }
}

pub fn random_species() -> Species {
    match rand::random::<u8>() % 5 {
        0 => Species::Red,
        1 => Species::Blue,
        2 => Species::Green,
        3 => Species::Yellow,
        4 => Species::Purple,
        _ => unreachable!(),
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub ball: Ball,
    pub collider: Collider,
    pub collision_types: ActiveCollisionTypes,
    pub species: Species,
}

impl BallBundle {
    pub fn new(
        pos: Vec3,
        radius: f32,
        species: Species,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: radius * BALL_RADIUS_COEFF,
                })),
                material: materials.add(species_to_color(species).into()),
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            collider: Collider::ball(radius * BALL_RADIUS_COEFF),
            species: species,
            ..Default::default()
        }
    }
}

impl Default for BallBundle {
    fn default() -> Self {
        BallBundle {
            pbr: Default::default(),
            ball: Ball,
            collider: Collider::ball(1.),
            collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            species: Species::Red,
        }
    }
}
