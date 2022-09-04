mod ball;
mod debug;
mod diagnostics;
mod game_over;
mod gameplay;
mod grid;
mod hex;
mod loading;
mod projectile;
mod start_menu;
mod utils;

use crate::debug::*;
use crate::diagnostics::*;
use crate::game_over::*;
use crate::gameplay::*;
use crate::grid::*;
use crate::loading::*;
use crate::projectile::*;
use crate::start_menu::*;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::window::WindowMode;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier3d::prelude::*;

pub const WINDOW_TITLE: &str = "ball shooter";

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
    app.add_plugin(RapierPhysicsPlugin::<()>::default());
    app.add_plugin(AudioPlugin);

    #[cfg(debug_assertions)]
    app.add_plugin(DiagnosticsPlugin);

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }

    // Plugins
    app.add_plugin(DebugPlugin);
    app.add_plugin(LoadingPlugin);
    app.add_plugin(ProjectilePlugin);
    app.add_plugin(GameplayPlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(StartMenuPlugin);
    app.add_plugin(GameOverPlugin);

    app.insert_resource(Msaa { samples: 4 });
    app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));
    app.insert_resource(WindowDescriptor {
        title: WINDOW_TITLE.to_string(),
        width: 1280.0,
        height: 720.0,
        position: WindowPosition::Automatic,
        scale_factor_override: Some(1.0), //Needed for some mobile devices, but disables scaling
        present_mode: PresentMode::AutoVsync,
        resizable: true,
        decorations: true,
        cursor_locked: false,
        cursor_visible: true,
        mode: WindowMode::Windowed,
        transparent: false,
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..Default::default()
    });
    app.add_state(AppState::Loading);
    app
}
