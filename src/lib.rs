mod diagnostics;
mod hex;
mod loading;
mod shoot;

use crate::diagnostics::*;
use crate::loading::*;
use crate::shoot::*;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

pub const WINDOW_TITLE: &str = "bevy app";

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum AppState {
    Loading,
    Next,
}

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(EguiPlugin);
    app.add_plugin(WorldInspectorPlugin::new());
    app.add_plugin(LookTransformPlugin);
    app.add_plugin(OrbitCameraPlugin::default());

    #[cfg(debug_assertions)]
    app.add_plugin(DiagnosticsPlugin);
    app.add_plugin(LoadingPlugin);
    app.add_plugin(ShootPlugin);

    app.insert_resource(Msaa { samples: 4 });
    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));
    app.insert_resource(WindowDescriptor {
        title: WINDOW_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        present_mode: bevy::window::PresentMode::AutoVsync,
        fit_canvas_to_parent: true,
        ..Default::default()
    });
    app.insert_resource(hex::Board {
        width: 10,
        height: 25,
        layout: hex::Layout {
            orientation: hex::Orientation::Pointy,
            origin: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
        },
        ..Default::default()
    });
    app.add_state(AppState::Loading);
    app.add_system_set(
        SystemSet::on_enter(AppState::Next)
            .with_system(setup_level)
            .with_system(setup_camera)
            .with_system(generate_board),
    );
    app.add_system_set(
        SystemSet::on_update(AppState::Next)
            .with_system(upkeep_hex_board)
            .with_system(rotate_balls),
    );
    app
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            radius: 50000.0,
            range: 50000.0,
            color: Color::rgb(1.0, 1.0, 1.0),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 15.0, 0.0),
        ..default()
    });
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0., 15., 0.),
            Vec3::new(0., 0., 0.),
        ));
}

fn generate_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut board: ResMut<hex::Board>,
) {
    for hex in hex::rectangle(&board) {
        let world_pos = board.hex_to_world_y(hex, 0.5);
        let entity = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    subdivisions: 1,
                    radius: 0.5,
                })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(world_pos),
                ..Default::default()
            })
            .insert(Name::new(format!("Hex {}, {}", hex.q, hex.r)))
            .insert(hex)
            .id();

        board.set(hex, Some(entity));
    }
}

fn upkeep_hex_board(
    mut hexes: Query<(Entity, &mut Transform, &hex::Hex), Changed<hex::Hex>>,
    mut board: ResMut<hex::Board>,
) {
    for (entity, mut transform, hex) in hexes.iter_mut() {
        let (x, z) = board.hex_to_world(*hex).into();
        transform.translation.x = x;
        transform.translation.z = z;
        board.set(*hex, Some(entity));
    }
}

fn rotate_balls(mut balls: Query<&mut Transform, With<hex::Hex>>) {
    for mut transform in balls.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(0.05);
    }
}
