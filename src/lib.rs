mod ball;
mod debug;
mod diagnostics;
mod gameplay;
mod grid;
mod hex;
mod loading;
mod projectile;
mod start_menu;
mod utils;

use crate::debug::*;
use crate::diagnostics::*;
use crate::gameplay::*;
use crate::grid::*;
use crate::loading::*;
use crate::projectile::*;
use crate::start_menu::*;

use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

pub const WINDOW_TITLE: &str = "bevy app";

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum AppState {
    Loading,
    Menu,
    Gameplay,
    GameOver,
}

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins_with(DefaultPlugins, |group| {
        group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
    });
    app.add_plugin(LookTransformPlugin);
    app.add_plugin(OrbitCameraPlugin::default());
    app.add_plugin(RapierPhysicsPlugin::<()>::default());

    #[cfg(debug_assertions)]
    app.add_plugin(DiagnosticsPlugin);

    app.add_plugin(DebugPlugin);
    app.add_plugin(LoadingPlugin);

    // Gameplay plugins
    app.add_plugin(ProjectilePlugin);
    app.add_plugin(GameplayPlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(StartMenuPlugin);

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
    app
}
