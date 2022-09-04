use bevy::{prelude::*, utils::hashbrown::hash_map};
use bevy_prototype_debug_lines::DebugLines;
use std::collections::{HashMap, HashSet};

use crate::loading::TextureAssets;

use super::{
    ball::{self, BallBundle},
    hex, AppState,
};

#[derive(Debug, Copy, Clone)]
pub struct GenerateGrid(pub i32, pub i32);

/// A dynamic hexagonal grid.
#[derive(Default, Debug, Clone)]
pub struct Grid {
    pub layout: hex::Layout,
    pub storage: HashMap<hex::Coord, Entity>,
    /// World bounds. Updated by calling [update_bounds].
    pub bounds: hex::Bounds,
    /// True if bounds haven't been updated since last modification.
    pub dirty: bool,
}

impl Grid {
    pub fn get(&self, hex: hex::Coord) -> Option<&Entity> {
        self.storage.get(&hex)
    }

    pub fn set(&mut self, hex: hex::Coord, entity: Option<Entity>) -> Option<Entity> {
        self.dirty = true;
        match entity {
            Some(entity) => self.storage.insert(hex.clone(), entity),
            None => self.storage.remove(&hex),
        }
    }

    pub fn dim(&self) -> (f32, f32) {
        (
            (self.bounds.mins.x - self.bounds.maxs.x).abs(),
            (self.bounds.mins.y - self.bounds.maxs.y).abs(),
        )
    }

    pub fn columns(&self) -> i32 {
        let (w, _) = self.dim();
        let (hw, _) = self.layout.hex_size();
        (w / hw / 2.).round() as i32
    }

    pub fn rows(&self) -> i32 {
        let (_, h) = self.dim();
        let (_, hh) = self.layout.hex_size();
        (h / hh / 2.).round() as i32
    }

    pub fn neighbors(&self, hex: hex::Coord) -> Vec<(hex::Coord, &Entity)> {
        hex.neighbors()
            .iter()
            .filter_map(|&hex| match self.get(hex) {
                Some(entity) => Some((hex, entity)),
                None => None,
            })
            .collect::<Vec<(hex::Coord, &Entity)>>()
    }

    // TODO: this is not that efficient, but should be fine for now.
    #[inline]
    pub fn update_bounds(&mut self) {
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        for (&hex, _) in self.storage.iter() {
            let pos = self.layout.to_world(hex);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
        }

        let (sx, sy) = self.layout.hex_size();

        self.dirty = false;
        self.bounds = hex::Bounds {
            mins: Vec2::new(min_x - sx, min_y - sy),
            maxs: Vec2::new(max_x + sx, max_y + sy),
        }
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.update_bounds();
    }
}

#[inline(always)]
pub fn find_cluster<'a, P>(
    grid: &Grid,
    origin: hex::Coord,
    is_cluster: P,
) -> (Vec<hex::Coord>, HashSet<hex::Coord>)
where
    P: Fn(&Entity) -> bool,
{
    let mut processed = HashSet::<hex::Coord>::new();
    let mut to_process = vec![origin];
    let mut cluster: Vec<hex::Coord> = vec![];

    processed.insert(origin);

    while let Some(current) = to_process.pop() {
        if let Some(entity) = grid.get(current) {
            if !is_cluster(entity) {
                continue;
            }

            cluster.push(current);

            for (hex, _) in grid.neighbors(current).iter() {
                if processed.contains(hex) {
                    continue;
                }
                to_process.push(*hex);
                processed.insert(*hex);
            }
        }
    }

    (cluster, processed)
}

#[inline(always)]
pub fn find_floating_clusters(grid: &Grid) -> Vec<Vec<hex::Coord>> {
    let mut processed = HashSet::<hex::Coord>::new();
    let mut floating_clusters: Vec<Vec<hex::Coord>> = vec![];

    for (hex, _) in grid.storage.iter() {
        if processed.contains(hex) {
            continue;
        }

        let (cluster, _processed) = find_cluster(grid, *hex, |_| true);

        processed.extend(_processed);

        if cluster.len() <= 0 {
            continue;
        }

        let mut floating = true;
        for hex in cluster.iter() {
            // TODO(pyrbin): we have to find a better way check if ball is top row
            if hex.r == 0 {
                floating = false;
                break;
            }
        }
        if floating {
            floating_clusters.push(cluster);
        }
    }
    floating_clusters
}

pub fn move_down_and_spawn(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: &mut Grid,
    texture_assets: &Res<TextureAssets>,
) {
    let mut hash_map: HashMap<hex::Coord, Option<&Entity>> = HashMap::new();
    for (&hex, entity) in grid.storage.iter() {
        let dir = match grid.layout.is_pointy() {
            true => match hex.r % 2 == 0 {
                true => hex::Direction::F,
                false => hex::Direction::E,
            },
            false => hex::Direction::F,
        };

        let down = hex.neighbor(dir);
        commands.entity(*entity).insert(down);
        hash_map.insert(down, Some(entity));
    }

    grid.storage = hash_map
        .iter()
        .map(|(&hex, &entity)| (hex, entity.unwrap().clone()))
        .collect();

    for hex in hex::rectangle(grid.columns(), 1, &grid.layout) {
        let world_pos = grid.layout.to_world_y(hex, 0.0);
        let ball = commands
            .spawn_bundle(BallBundle::new(
                world_pos,
                grid.layout.size.x,
                ball::random_species(),
                &mut meshes,
                &mut materials,
                texture_assets,
            ))
            .insert(hex)
            .id();

        grid.set(hex, Some(ball));
    }
}

fn generate_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<Grid>,
    hexes: Query<Entity, With<hex::Coord>>,
    texture_assets: Res<TextureAssets>,
) {
    for entity in hexes.iter() {
        commands.entity(entity).despawn();
    }

    grid.clear();

    const WIDTH: i32 = 16;
    const HEIGHT: i32 = 16;

    for hex in hex::rectangle(WIDTH, HEIGHT, &grid.layout) {
        let world_pos = grid.layout.to_world_y(hex, 0.0);
        let entity = commands
            .spawn_bundle(BallBundle::new(
                world_pos,
                grid.layout.size.x,
                ball::random_species(),
                &mut meshes,
                &mut materials,
                &texture_assets,
            ))
            .insert(hex)
            .id();

        grid.set(hex, Some(entity));
    }

    grid.update_bounds();

    // Center grid on x-axis.
    let (width, _) = grid.dim();
    grid.layout.origin.x = -width / 2.;

    grid.update_bounds();
}

fn update_hex_coord_transforms(
    mut hexes: Query<(Entity, &mut Transform, &hex::Coord), Changed<hex::Coord>>,
    mut grid: ResMut<Grid>,
) {
    for (entity, mut transform, hex) in hexes.iter_mut() {
        grid.set(*hex, Some(entity));
        let (x, z) = grid.layout.to_world(*hex).into();
        transform.translation.x = x;
        transform.translation.z = z;
    }
}

fn display_grid_bounds(grid: Res<Grid>, mut lines: ResMut<DebugLines>) {
    const Z_LENGTH: f32 = 1000.;

    lines.line_colored(
        Vec3::new(grid.bounds.mins.x, 0., Z_LENGTH),
        Vec3::new(grid.bounds.mins.x, 0., -Z_LENGTH),
        0.,
        Color::GRAY,
    );

    lines.line_colored(
        Vec3::new(grid.bounds.maxs.x, 0., Z_LENGTH),
        Vec3::new(grid.bounds.maxs.x, 0., -Z_LENGTH),
        0.,
        Color::GRAY,
    );
}

fn cleanup_grid(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    hexes: Query<Entity, With<hex::Coord>>,
) {
    for entity in hexes.iter() {
        commands.entity(entity).despawn();
    }
    grid.clear();
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            layout: hex::Layout {
                orientation: hex::Orientation::pointy().clone(),
                origin: Vec2::new(0.0, 0.0),
                size: Vec2::new(1.0, 1.0),
            },
            ..Default::default()
        });
        app.add_system_set(SystemSet::on_enter(AppState::Gameplay).with_system(generate_grid));
        app.add_system_set(
            SystemSet::on_update(AppState::Gameplay).with_system(update_hex_coord_transforms),
        );
        app.add_system_set(
            SystemSet::on_update(AppState::Gameplay).with_system(display_grid_bounds),
        );
        app.add_system_set(SystemSet::on_exit(AppState::Gameplay).with_system(cleanup_grid));
    }
}
