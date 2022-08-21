mod diagnostics;
mod loading;

use crate::diagnostics::*;
use crate::loading::*;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub const WINDOW_TITLE: &str = "bevy app";

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Playing,
    Menu,
}

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(LoadingPlugin);
    app.add_plugin(EguiPlugin);
    #[cfg(debug_assertions)]
    app.add_plugin(DiagnosticsPlugin);
    app.insert_resource(Msaa { samples: 1 });
    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));
    app.insert_resource(WindowDescriptor {
        title: WINDOW_TITLE.to_string(),
        canvas: Some("#bevy".to_string()),
        present_mode: bevy::window::PresentMode::AutoNoVsync,
        fit_canvas_to_parent: true,
        ..Default::default()
    });
    app.add_state(GameState::Loading);
    app.add_system_set(SystemSet::on_enter(GameState::Menu).with_system(load_icon));
    app
}

fn load_icon(mut commands: Commands, texture_assets: Res<TextureAssets>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: texture_assets.texture_bevy.clone(),
        ..default()
    });
}
