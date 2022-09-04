use std::time::Duration;

use crate::loading::{AudioAssets, FontAssets};
use crate::AppState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

struct ButtonColors {
    normal: UiColor,
    hovered: UiColor,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
        }
    }
}

struct SoundtrackAudio(Handle<AudioInstance>);

fn start_audio(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    audio.pause();
    let handle = audio
        .play(audio_assets.soundtrack.clone())
        .looped()
        .fade_in(AudioTween::linear(Duration::from_secs(5)))
        .with_volume(0.4)
        .handle();

    commands.insert_resource(SoundtrackAudio(handle));
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: button_colors.normal,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Play".to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    }],
                    alignment: Default::default(),
                },
                ..Default::default()
            });
        });
}

fn click_play_button(
    button_colors: Res<ButtonColors>,
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(AppState::Gameplay).unwrap();
            }
            Interaction::Hovered => {
                *color = button_colors.hovered;
            }
            Interaction::None => {
                *color = button_colors.normal;
            }
        }
    }
}

fn cleanup_menu(
    mut commands: Commands,
    button: Query<Entity, With<Button>>,
    cam: Query<Entity, With<Camera2d>>,
) {
    commands.entity(button.single()).despawn_recursive();
    commands.entity(cam.single()).despawn_recursive();
}

pub struct StartMenuPlugin;

impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system_set(
                SystemSet::on_enter(AppState::Menu)
                    .with_system(setup_menu)
                    .with_system(start_audio),
            )
            .add_system_set(SystemSet::on_update(AppState::Menu).with_system(click_play_button))
            .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu));
    }
}
