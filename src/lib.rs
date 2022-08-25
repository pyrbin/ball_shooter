mod ball;
mod debug;
mod diagnostics;
mod grid;
mod hex;
mod loading;
mod shoot;
mod utils;

use crate::diagnostics::*;
use crate::grid::*;
use crate::loading::*;
use crate::shoot::*;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
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

#[derive(Component)]
pub struct MainCamera;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(LookTransformPlugin);
    app.add_plugin(OrbitCameraPlugin::default());
    app.add_plugin(RapierPhysicsPlugin::<()>::default());
    // Debugging
    app.add_plugin(EguiPlugin);
    app.add_plugin(WorldInspectorPlugin::new());

    //app.add_plugin(RapierDebugRenderPlugin::default());
    app.add_plugin(DebugLinesPlugin::with_depth_test(true));

    app.add_plugin(DiagnosticsPlugin);

    app.add_plugin(LoadingPlugin);
    app.add_plugin(ShootPlugin);
    app.add_plugin(GridPlugin);

    app.insert_resource(Msaa { samples: 4 });
    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));
    app.insert_resource(WindowDescriptor {
        title: WINDOW_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        present_mode: bevy::window::PresentMode::AutoVsync,
        fit_canvas_to_parent: true,
        ..Default::default()
    });
    app.add_state(AppState::Loading);
    app.add_system_set(
        SystemSet::on_enter(AppState::Next)
            .with_system(setup_level)
            .with_system(setup_camera),
    );
    app
}

fn setup_level(mut commands: Commands) {
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
        ))
        .insert(MainCamera);
}
