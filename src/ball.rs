use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::hex;

pub const BALL_RADIUS_COEFF: f32 = hex::INNER_RADIUS_COEFF * 0.85;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct Species(pub Color);

#[derive(Bundle)]
pub struct BallBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub ball: Ball,
    pub collider: Collider,
    pub collision_types: ActiveCollisionTypes,
}

impl BallBundle {
    pub fn new(
        pos: Vec3,
        radius: f32,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: radius * BALL_RADIUS_COEFF,
                })),
                material: materials.add(Color::rgba(0.8, 0.7, 0.6, 0.8).into()),
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            collider: Collider::ball(radius * BALL_RADIUS_COEFF),
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
        }
    }
}
