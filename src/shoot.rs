use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    ball::{self, BallBundle},
    hex, AppState,
};

#[derive(Component, Clone, Default)]
pub struct Projectile;

pub struct BallHitEvent(pub Entity);

fn fire_projectile(
    mut commands: Commands,
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    projectile: Query<(Entity, &Projectile)>,
    keys: Res<Input<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    for (entity, _) in projectile.iter() {
        commands.entity(entity).despawn();
    }

    for (camera, camera_transform) in cameras.iter() {
        let (ray_pos, ray_dir) =
            ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

        let (plane_pos, plane_normal) = (Vec3::new(0., 0.5, 0.), Vec3::Y);
        let point = plane_intersection(ray_pos, ray_dir, plane_pos, plane_normal);

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: 0.75,
                })),
                material: materials.add(Color::rgb(1., 0.1, 0.1).into()),
                transform: Transform::from_translation(point),
                ..Default::default()
            })
            .insert(Projectile)
            .insert(RigidBody::KinematicVelocityBased)
            .insert(Velocity::linear(Vec3::new(0., 0., -15.)))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Collider::ball(0.75));
    }
}

/// Calculate the intersection point of a vector and a plane defined as a point and normal vector
/// where `pv` is the vector point, `dv` is the vector direction,
/// `pp` is the plane point and `np` is the planes' normal vector
fn plane_intersection(pv: Vec3, dv: Vec3, pp: Vec3, np: Vec3) -> Vec3 {
    let d = dv.dot(np);
    let t = (pp.dot(np) - pv.dot(np)) / d;
    pv + dv * t
}

fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

    let camera_inverse_matrix =
        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (near, dir)
}

fn check_projectile_collisions(
    mut events: EventReader<CollisionEvent>,
    mut ball_hit: EventWriter<BallHitEvent>,
    projectile: Query<Entity, With<Projectile>>,
    balls: Query<Entity, With<ball::Ball>>,
) {
    for event in events.iter() {
        match event {
            CollisionEvent::Started(d1, d2, _) => {
                if let Ok(_) = projectile.get(*d1).or(projectile.get(*d2)) {
                    if let Ok(ball) = balls.get(*d1).or(balls.get(*d2)) {
                        ball_hit.send(BallHitEvent(ball));
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
    }
}

fn on_ball_hit_event(
    mut ball_hit: EventReader<BallHitEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    projectile: Query<(Entity, &Transform), With<Projectile>>,
    mut board: ResMut<hex::Board>,
) {
    for _ in ball_hit.iter().take(1) {
        if let Ok((entity, transform)) = projectile.get_single() {
            let hex = board.world_to_hex(transform.translation);
            let world_pos = board.hex_to_world_y(hex, 0.5);
            let ball = if let Some(ball) = board.get(hex) {
                *ball
            } else {
                commands
                    .spawn_bundle(BallBundle {
                        pbr: PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Icosphere {
                                subdivisions: 1,
                                radius: 0.75,
                            })),
                            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                            transform: Transform::from_translation(world_pos),
                            ..Default::default()
                        },
                        collider: Collider::ball(0.75),
                        ..Default::default()
                    })
                    .insert(hex)
                    .id()
            };

            board.set(hex, Some(ball));
            commands.entity(entity).despawn();
        }
    }
}

pub struct ShootPlugin;

impl Plugin for ShootPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BallHitEvent>();
        app.add_system_set(
            SystemSet::on_update(AppState::Next)
                .with_system(fire_projectile)
                .with_system(check_projectile_collisions)
                .with_system(on_ball_hit_event),
        );
    }
}
