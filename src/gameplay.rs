use crate::{
    ball, grid, hex,
    loading::{AudioAssets, FontAssets, TextureAssets},
    projectile, AppState,
};
use bevy::{prelude::*, render::camera::Projection};
use bevy_kira_audio::prelude::*;
use bevy_mod_check_filter::IsTrue;
use bevy_prototype_debug_lines::DebugLines;

#[derive(Component)]
pub struct MainCamera;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Score(pub u32);

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct TurnCounter(pub u32);

#[derive(Debug, Clone)]
pub struct BeginTurn;

pub const PLAYER_SPAWN_Z: f32 = 40.0;

fn setup_gameplay(
    mut begin_turn: EventWriter<BeginTurn>,
    mut turn_counter: ResMut<TurnCounter>,
    mut score: ResMut<Score>,
) {
    score.0 = 0;
    turn_counter.0 = 0;
    begin_turn.send(BeginTurn);
}

fn on_begin_turn(mut turn_counter: ResMut<TurnCounter>, begin_turn: EventReader<BeginTurn>) {
    if begin_turn.is_empty() {
        return;
    }
    begin_turn.clear();
    turn_counter.0 += 1;
}

fn on_snap_projectile(
    snap_projectile: EventReader<projectile::SnapProjectile>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<grid::Grid>,
    mut begin_turn: EventWriter<BeginTurn>,
    mut score: ResMut<Score>,
    turn_counter: ResMut<TurnCounter>,
    projectile: Query<
        (Entity, &Transform, &ball::Species),
        (With<projectile::Projectile>, IsTrue<projectile::Flying>),
    >,
    balls: Query<&ball::Species, With<ball::Ball>>,
    texture_assets: Res<TextureAssets>,
    audio: Res<bevy_kira_audio::Audio>,
    audio_assets: Res<AudioAssets>,
) {
    if snap_projectile.is_empty() {
        return;
    }

    // We really only care about the first ball hit event
    snap_projectile.clear();

    if let Ok((entity, tr, species)) = projectile.get_single() {
        commands.entity(entity).despawn();

        let y = tr.translation.y;
        let mut translation = tr.translation;
        let mut hex = grid.layout.from_world(translation);

        // hard check to make sure the projectile is inside the grid bounds.
        let (hex_radius, _) = grid.layout.hex_size();
        const SKIN_WIDTH: f32 = 0.1;
        let radius = hex_radius + SKIN_WIDTH;
        let (clamped, was_clamped, _) = projectile::clamp_inside_world_bounds(
            grid.layout.to_world_y(hex, y),
            radius,
            &grid.bounds,
        );
        if was_clamped {
            hex = grid.layout.from_world(clamped);
        }

        // Dumb iterative check to make sure chosen hex is not occupied.
        const MAX_ITER: usize = 10;
        let mut iter = 0;
        while let Some(_) = grid.get(hex) {
            let step_size = Vec3::Z * radius;
            translation += step_size;
            (translation, _, _) =
                projectile::clamp_inside_world_bounds(translation, radius, &grid.bounds);

            hex = grid.layout.from_world(translation);

            iter += 1;
            if iter >= MAX_ITER {
                break;
            }
        }

        let final_pos = grid.layout.to_world_y(hex, y);
        let ball = commands
            .spawn_bundle(ball::BallBundle::new(
                final_pos,
                grid.layout.size.x,
                *species,
                &mut meshes,
                &mut materials,
                &texture_assets,
            ))
            .insert(hex)
            .id();

        grid.set(hex, Some(ball));

        let (cluster, _) = grid::find_cluster(&grid, hex, |&e| {
            e == ball
                || match balls.get(e) {
                    Ok(other) => *other == *species,
                    Err(_) => false,
                }
        });

        let mut score_add = 0;

        // remove matching clusters
        const MIN_CLUSTER_SIZE: usize = 3;
        if cluster.len() >= MIN_CLUSTER_SIZE {
            cluster.iter().for_each(|&hex| {
                commands.entity(*grid.get(hex).unwrap()).despawn();
                grid.set(hex, None);
                score_add += 1;
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
                score_add += 1;
            });

        const MOVE_DOWN_TURN: u32 = 5;
        if turn_counter.0 % MOVE_DOWN_TURN == 0 {
            grid::move_down_and_spawn(
                &mut commands,
                meshes,
                materials,
                grid.as_mut(),
                &texture_assets,
            );
        }

        // remove floating clusters
        let floating_clusters = grid::find_floating_clusters(&grid);
        floating_clusters
            .iter()
            .flat_map(|e| e.iter())
            .for_each(|&hex| {
                commands.entity(*grid.get(hex).unwrap()).despawn();
                grid.set(hex, None);
                score_add += 1;
            });

        if score_add > 0 {
            audio.play(audio_assets.score.clone());
        }

        score.0 += score_add;

        begin_turn.send(BeginTurn);
    }
}

fn check_game_over(
    grid: Res<grid::Grid>,
    mut app_state: ResMut<State<AppState>>,
    mut lines: ResMut<DebugLines>,
) {
    let projectile_hex = grid.layout.from_world(Vec3::new(0.0, 0.0, PLAYER_SPAWN_Z));
    let game_over_row = projectile_hex
        .neighbor(hex::Direction::B)
        .neighbor(hex::Direction::B);
    let row_pos = grid.layout.to_world_y(game_over_row, 0.0);

    lines.line_colored(
        Vec3::new(grid.bounds.mins.x, 0., row_pos.z),
        Vec3::new(grid.bounds.maxs.x, 0., row_pos.z),
        0.,
        Color::RED,
    );

    for (&hex, _) in grid.storage.iter() {
        let world_pos = grid.layout.to_world_y(hex, 0.0);
        if world_pos.z >= row_pos.z - 0.1 {
            app_state.set(AppState::GameOver).unwrap();
            break;
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 76.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 70.0, 41.0)
                .looking_at(Vec3::new(0.0, 0.0, PLAYER_SPAWN_Z / 2.), Vec3::Y),
            ..default()
        })
        .insert(MainCamera);
}

fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>, score: Res<Score>) {
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: format!(" Score: {:?} ", score.0).to_string(),
                style: TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            }],
            alignment: Default::default(),
        },
        transform: Transform::from_xyz(0.0, 100.0, 0.0),
        ..Default::default()
    });
}

fn update_ui(score: Res<Score>, mut score_text: Query<&mut Text>) {
    for mut text in &mut score_text {
        text.sections[0].value = format!(" Score: {:?} ", score.0);
    }
}

fn cleanup_gameplay(
    mut commands: Commands,
    camera: Query<Entity, With<MainCamera>>,
    score_text: Query<Entity, With<Text>>,
) {
    commands.entity(camera.single()).despawn_recursive();
    commands.entity(score_text.single()).despawn_recursive();
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BeginTurn>();
        app.insert_resource(TurnCounter(0));
        app.insert_resource(Score(0));
        app.add_system_set(
            SystemSet::on_enter(AppState::Gameplay)
                .with_system(setup_ui)
                .with_system(setup_camera)
                .with_system(setup_gameplay),
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Gameplay)
                .with_system(update_ui)
                .with_system(on_begin_turn)
                .with_system(check_game_over)
                .with_system(on_snap_projectile),
        );
        app.add_system_set(SystemSet::on_exit(AppState::Gameplay).with_system(cleanup_gameplay));
    }
}
