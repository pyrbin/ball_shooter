use bevy::prelude::*;
use bevy_mod_check_filter::{IsFalse, IsTrue};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;

use super::{
    ball::{self, BallBundle, Species},
    grid, utils, AppState, MainCamera,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum ProjectileStage {
    PostPhysics,
}

#[derive(Component, Clone, Default)]
pub struct Projectile;

#[derive(Clone)]
pub struct BallHitEvent {
    /// Entity of the ball that was hit.
    pub entity: Entity,
    /// Hit normal outwards from the projectile.
    pub hit_normal: Vec3,
}

#[derive(Component, Clone, Default)]
pub struct ReloadProjectile;

#[derive(Component)]
pub struct Flying(pub bool);

impl std::ops::Deref for Flying {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    ) -> Self {
        Self {
            pbr: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: radius * ball::BALL_RADIUS_COEFF,
                })),
                material: materials.add(ball::species_to_color(species).into()),
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

fn setup_shooting(mut commands: Commands) {
    commands.spawn().insert(ReloadProjectile);
}

fn projectile_reload(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    trigger: Query<Entity, With<ReloadProjectile>>,
    grid: Res<grid::Grid>,
) {
    if let Ok(trigger) = trigger.get_single() {
        commands.entity(trigger).despawn();
        commands.spawn_bundle(ProjectileBundle::new(
            Vec3::new(0.0, 0.0, 40.0),
            grid.layout.size.x,
            ball::random_species(),
            &mut meshes,
            &mut materials,
        ));
    }
}

fn move_down_and_spawn(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<grid::Grid>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        grid::move_down_and_spawn(commands, meshes, materials, grid.as_mut());
    }
}

fn aim_projectile(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut projectile: Query<(Entity, &Transform, &mut Velocity, &mut Flying), IsFalse<Flying>>,
    mouse: Res<Input<MouseButton>>,
    mut lines: ResMut<DebugLines>,
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

        // TODO(pyrbin): use angle instead
        const Z_OFFSET: f32 = -1.;
        point.z = point.z.min(transform.translation.z + Z_OFFSET);

        lines.line_colored(transform.translation, point, 0.0, Color::GREEN);

        if !mouse.just_pressed(MouseButton::Left) {
            return;
        }

        const PROJECTILE_SPEED: f32 = 50.;
        let aim_direction = (point - transform.translation).normalize();
        vel.linvel = aim_direction * PROJECTILE_SPEED;

        is_flying.0 = true;
    }
}

fn bounce_on_world_bounds(
    mut projectile: Query<(Entity, &mut Transform, &mut Velocity, &Collider), IsTrue<Flying>>,
    grid: Res<grid::Grid>,
) {
    if let Ok((_, mut transform, mut vel, collider)) = projectile.get_single_mut() {
        if let Some(shape) = collider.raw.as_ball() {
            const SKIN_WIDTH: f32 = 0.1;
            let skin = shape.radius + SKIN_WIDTH;

            let (clamped, was_clamped) =
                clamp_inside_world_bounds(transform.translation, skin, grid.bounds.0);

            transform.translation = clamped;

            if was_clamped {
                vel.linvel.x = -vel.linvel.x;
            }
        }
    }
}

fn clamp_inside_world_bounds(mut pos: Vec3, size: f32, x_bounds: Vec2) -> (Vec3, bool) {
    let x = pos.x;
    let mut clamped = false;
    let (x0, x1) = (x - size, x + size);
    if x0 <= x_bounds.x {
        pos.x = x_bounds.x + size;
        clamped = true;
    } else if x1 >= x_bounds.y {
        pos.x = x_bounds.y - size;
        clamped = true;
    }
    (pos, clamped)
}

fn on_projectile_collisions_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut ball_hit_write: EventWriter<BallHitEvent>,
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
            ball_hit_write.send(BallHitEvent { entity, hit_normal });
        }
    }
}

fn on_ball_hit_event(
    ball_hits: EventReader<BallHitEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<grid::Grid>,
    projectile: Query<(Entity, &Transform, &ball::Species), (With<Projectile>, IsTrue<Flying>)>,
    balls: Query<&ball::Species, With<ball::Ball>>,
) {
    if ball_hits.is_empty() {
        return;
    }

    if let Ok((entity, tr, species)) = projectile.get_single() {
        let y = tr.translation.y;
        let mut translation = tr.translation;
        let mut hex = grid.hex_coords(translation);

        // hard check to make sure the projectile is inside the grid bounds.
        let (hex_radius, _) = grid.hex_world_size();
        const SKIN_WIDTH: f32 = 0.1;
        let radius = hex_radius + SKIN_WIDTH;
        let (clamped, was_clamped) =
            clamp_inside_world_bounds(grid.world_pos_y(hex, y), radius, grid.bounds.0);
        if was_clamped {
            hex = grid.hex_coords(clamped);
        }

        // Dumb iterative check to make sure chosen hex is not occupied.
        const MAX_ITER: usize = 10;
        let mut iter = 0;
        while let Some(_) = grid.get(hex) {
            let step_size = Vec3::Z * radius;
            translation += step_size;
            (translation, _) = clamp_inside_world_bounds(translation, radius, grid.bounds.0);

            hex = grid.hex_coords(translation);

            iter += 1;
            if iter >= MAX_ITER {
                break;
            }
        }

        commands.entity(entity).despawn();
        commands.spawn().insert(ReloadProjectile);

        let final_pos = grid.world_pos_y(hex, y);
        let ball = commands
            .spawn_bundle(BallBundle::new(
                final_pos,
                grid.layout.size.x,
                *species,
                &mut meshes,
                &mut materials,
            ))
            .insert(hex)
            .insert(Name::new(
                format!("Hex {:?}, {:?}", hex.q, hex.r).to_string(),
            ))
            .id();

        grid.set(hex, Some(ball));

        let (cluster, _) = grid::find_cluster(&grid, hex, |e| match *e == ball {
            true => true,
            false => match balls.get(*e) {
                Ok(other) => other == species,
                Err(_) => false,
            },
        });

        // remove matching clusters
        const MIN_CLUSTER_SIZE: usize = 3;
        if cluster.len() >= MIN_CLUSTER_SIZE {
            cluster.iter().for_each(|&hex| {
                commands.entity(*grid.get(hex).unwrap()).despawn();
                grid.set(hex, None);
            });
        }

        // remove floating clusters
        let floating_clusters = grid::find_floating_clusters(&grid);
        floating_clusters
            .iter()
            .flat_map(|e| e.iter())
            .for_each(|&hex| {
                commands.entity(*grid.get(hex).unwrap()).despawn();
                grid.set(hex, None);
            });
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BallHitEvent>();
        app.add_system_set(SystemSet::on_enter(AppState::Next).with_system(setup_shooting));
        app.add_system_set(
            SystemSet::on_update(AppState::Next)
                .with_system(move_down_and_spawn)
                .with_system(projectile_reload)
                .with_system(aim_projectile),
        );
        app.add_stage_before(
            PhysicsStages::SyncBackend,
            ProjectileStage::PostPhysics,
            SystemStage::single_threaded(),
        );
        app.add_system_set_to_stage(
            ProjectileStage::PostPhysics,
            SystemSet::new()
                .with_system(bounce_on_world_bounds)
                .with_system(on_projectile_collisions_events)
                .with_system(on_ball_hit_event),
        );
    }
}
