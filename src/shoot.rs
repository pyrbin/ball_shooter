use std::ops::Add;

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::AppState;

#[derive(Component, Clone, Default)]
pub struct Projectile {
    linvel: Vec2,
}

#[derive(Component, Clone, Default)]
pub struct Flying;

fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 1,
                radius: 0.5,
            })),
            material: materials.add(Color::rgb(1., 0.1, 0.1).into()),
            transform: Transform::from_xyz(0., 0.5, 20.),
            ..Default::default()
        })
        .insert(Projectile {
            linvel: Vec2::new(0.0, -5.),
        });
}

fn fire_projectile(
    mut commands: Commands,
    mut projectile: Query<(Entity, &Projectile), Without<Flying>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for (entity, _) in projectile.iter_mut() {
            commands.entity(entity).insert(Flying);
        }
    }
}

fn update_projectile(
    mut projectile: Query<(&mut Transform, &Projectile), With<Flying>>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in projectile.iter_mut() {
        transform.translation += Vec3::new(
            projectile.linvel.x * time.delta_seconds(),
            0.,
            projectile.linvel.y * time.delta_seconds(),
        );
    }
}

pub struct ShootPlugin;

impl Plugin for ShootPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Next).with_system(spawn_projectile));
        app.add_system_set(
            SystemSet::on_update(AppState::Next)
                .with_system(fire_projectile)
                .with_system(update_projectile),
        );
    }
}
