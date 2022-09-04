use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_mod_check_filter::{IsFalse, IsTrue};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;

use crate::{
    gameplay, hex,
    loading::{AudioAssets, TextureAssets},
};

use super::{
    ball::{self, Species},
    grid, utils, AppState, MainCamera,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum ProjectileStage {
    Update,
}

#[derive(Component, Clone, Default)]
pub struct Projectile;

#[derive(Component)]
pub struct Flying(pub bool);

impl std::ops::Deref for Flying {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct SnapProjectile {
    /// Entity of the ball if any were hit.
    pub entity: Option<Entity>,
    /// Hit normal outwards from the projectile if any ball were hit.
    pub hit_normal: Option<Vec3>,
}

#[derive(Clone)]
pub struct SpawnedBall {
    pub hex: hex::Coord,
    pub species: ball::Species,
}

#[derive(Clone)]
pub struct ReloadProjectile;

#[derive(Clone)]
pub struct ProjectileBuffer(pub Vec<ball::Species>);

/// We apply a tiny reduction to the projectile collider radius.
pub const PROJ_COLLIDER_COEFF: f32 = 0.783;

#[derive(Bundle)]
pub struct ProjectileBundle {
    #[bundle]
    pub pbr: PbrBundle,
    pub rigid_body: RigidBody,
    pub ccd: Ccd,
    pub collider: Collider,
    pub velocity: Velocity,
    pub collision_events: ActiveEvents,
    pub projectile: Projectile,
    pub is_flying: Flying,
    pub species: Species,
}

impl ProjectileBundle {
    pub fn new(
        pos: Vec3,
        radius: f32,
        species: Species,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        texture_assets: &Res<TextureAssets>,
    ) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: radius * ball::BALL_RADIUS_COEFF,
                })),
                material: materials.add(StandardMaterial {
                    base_color: ball::species_to_color(species).into(),
                    base_color_texture: Some(texture_assets.texture_bevy.clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            collider: Collider::ball(radius * ball::BALL_RADIUS_COEFF * PROJ_COLLIDER_COEFF),
            is_flying: Flying(false),
            species: species,
            ..Default::default()
        }
    }
}

impl Default for ProjectileBundle {
    fn default() -> Self {
        ProjectileBundle {
            pbr: Default::default(),
            rigid_body: RigidBody::KinematicVelocityBased,
            collider: Collider::ball(1.),
            collision_events: ActiveEvents::all(),
            projectile: Projectile,
            is_flying: Flying(false),
            velocity: Velocity::linear(Vec3::new(0., 0., 0.)),
            ccd: Ccd::enabled(),
            species: Species::Red,
        }
    }
}

fn projectile_reload(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut buffer: ResMut<ProjectileBuffer>,
    begin_turn: EventReader<gameplay::BeginTurn>,
    grid: Res<grid::Grid>,
    texture_assets: Res<TextureAssets>,
) {
    if begin_turn.is_empty() {
        return;
    }

    begin_turn.clear();

    let species = match buffer.0.pop() {
        Some(species) => species,
        None => ball::random_species(),
    };

    commands.spawn_bundle(ProjectileBundle::new(
        Vec3::new(0.0, 0.0, gameplay::PLAYER_SPAWN_Z),
        grid.layout.size.x,
        species,
        &mut meshes,
        &mut materials,
        &texture_assets,
    ));

    buffer.0.push(ball::random_species());
}

fn aim_projectile(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut projectile: Query<(Entity, &Transform, &mut Velocity, &mut Flying), IsFalse<Flying>>,
    mouse: Res<Input<MouseButton>>,
    mut lines: ResMut<DebugLines>,
    audio: Res<bevy_kira_audio::Audio>,
    audio_assets: Res<AudioAssets>,
) {
    if let Ok((_, transform, mut vel, mut is_flying)) = projectile.get_single_mut() {
        let (camera, camera_transform) = cameras.single();
        let (ray_pos, ray_dir) = utils::ray_from_mouse_position(
            windows.get_primary().unwrap(),
            camera,
            camera_transform,
        );
        let (plane_pos, plane_normal) = (Vec3::new(0., transform.translation.y, 0.), Vec3::Y);

        let mut point = utils::plane_intersection(ray_pos, ray_dir, plane_pos, plane_normal);
        point.y = 0.0;

        // should use an angle instead
        point.z = point.z.min(transform.translation.z - 5.);

        lines.line_colored(transform.translation, point, 0.0, Color::GREEN);

        if !mouse.just_pressed(MouseButton::Left) {
            return;
        }

        audio.play(audio_assets.flying.clone());

        const PROJECTILE_SPEED: f32 = 30.;
        let aim_direction = (point - transform.translation).normalize();
        vel.linvel = aim_direction * PROJECTILE_SPEED;

        is_flying.0 = true;
    }
}

fn bounce_on_world_bounds(
    mut projectile: Query<(Entity, &mut Transform, &mut Velocity, &Collider), IsTrue<Flying>>,
    mut snap_projectile: EventWriter<SnapProjectile>,
    grid: Res<grid::Grid>,
) {
    if let Ok((_, mut transform, mut vel, collider)) = projectile.get_single_mut() {
        if let Some(shape) = collider.raw.as_ball() {
            const SKIN_WIDTH: f32 = 0.1;
            let skin = shape.radius + SKIN_WIDTH;

            let (clamped, was_clamped_x, was_clamped_y) =
                clamp_inside_world_bounds(transform.translation, skin, &grid.bounds);

            transform.translation = clamped;

            if was_clamped_x {
                vel.linvel.x = -vel.linvel.x;
            }

            // We hit the top, snap ball
            if was_clamped_y {
                vel.linvel = Vec3::ZERO;
                snap_projectile.send(SnapProjectile {
                    entity: None,
                    hit_normal: None,
                });
            }
        }
    }
}

pub fn clamp_inside_world_bounds(
    mut pos: Vec3,
    size: f32,
    grid_bounds: &hex::Bounds,
) -> (Vec3, bool, bool) {
    let (x, _, y) = pos.into();

    let mut clamped_x = false;
    let mut clamped_y = false;

    let (x0, x1) = (x - size, x + size);
    let y1 = y - size;

    if x0 <= grid_bounds.mins.x {
        pos.x = grid_bounds.mins.x + size;
        clamped_x = true;
    } else if x1 >= grid_bounds.maxs.x {
        pos.x = grid_bounds.maxs.x - size;
        clamped_x = true;
    }

    if y1 <= grid_bounds.mins.y {
        pos.y = grid_bounds.mins.y - size;
        clamped_y = true;
    }

    (pos, clamped_x, clamped_y)
}

fn on_projectile_collisions_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut snap_projectile: EventWriter<SnapProjectile>,
    mut projectile: Query<(Entity, &mut Velocity, &Transform), (With<Projectile>, IsTrue<Flying>)>,
    balls: Query<(Entity, &Transform), With<ball::Ball>>,
) {
    for (d1, d2, _) in collision_events.iter().filter_map(|e| match e {
        CollisionEvent::Started(a, b, f) => Some((a, b, f)),
        CollisionEvent::Stopped(_, _, _) => None,
    }) {
        let mut p1 = projectile.get_mut(*d1);
        if p1.is_err() {
            p1 = projectile.get_mut(*d2);
        }

        if let Ok((entity, otr)) = balls.get(*d1).or(balls.get(*d2)) {
            let (_, mut vel, tr) = p1.unwrap();
            let hit_normal = (otr.translation - tr.translation).normalize();
            vel.linvel = Vec3::ZERO;
            snap_projectile.send(SnapProjectile {
                entity: Some(entity),
                hit_normal: Some(hit_normal),
            });
        }
    }
}

fn rotate_projectile(
    mut query: Query<(Entity, &mut Transform), (With<Projectile>, IsTrue<Flying>)>,
) {
    for (_, mut transform) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(0.1);
    }
}

fn cleanup_projectile(mut commands: Commands, projectile: Query<Entity, With<Projectile>>) {
    if !projectile.iter().next().is_none() {
        commands.entity(projectile.single()).despawn_recursive();
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SnapProjectile>();
        app.add_event::<SpawnedBall>();
        app.insert_resource(ProjectileBuffer(vec![ball::random_species()]));
        app.add_system_set(
            SystemSet::on_update(AppState::Gameplay)
                .with_system(rotate_projectile)
                .with_system(projectile_reload)
                .with_system(aim_projectile),
        );
        app.add_stage_before(
            PhysicsStages::SyncBackend,
            ProjectileStage::Update,
            SystemStage::single_threaded(),
        );
        app.add_system_set_to_stage(
            ProjectileStage::Update,
            SystemSet::new()
                .with_system(bounce_on_world_bounds)
                .with_system(on_projectile_collisions_events),
        );
        app.add_system_set(SystemSet::on_exit(AppState::Gameplay).with_system(cleanup_projectile));
    }
}
