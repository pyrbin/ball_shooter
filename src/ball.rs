use bevy::{ecs::bundle, prelude::*};
use bevy_rapier3d::prelude::*;

#[derive(Bundle)]
pub struct BallBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub ball: Ball,
    pub collider: Collider,
    pub collision_types: ActiveCollisionTypes,
    pub sensor: Sensor,
}

impl Default for BallBundle {
    fn default() -> Self {
        BallBundle {
            pbr: Default::default(),
            ball: Ball,
            collider: Collider::ball(1.),
            collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            sensor: Sensor,
        }
    }
}

#[derive(Component)]
pub struct Ball;
