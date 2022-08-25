use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use std::collections::HashMap;

use crate::{ball::BallBundle, hex, AppState};

#[derive(Debug, Copy, Clone)]
pub struct GenerateGrid(pub i32, pub i32);

/// A dynamic hexagonal grid.
#[derive(Default, Debug, Clone)]
pub struct Grid {
    pub layout: hex::Layout,
    pub storage: HashMap<hex::Hex, Entity>,
    /// World bounds.
    /// `0` = (min_x, max_x),
    /// `1` = (min_y, max_y).
    pub bounds: (Vec2, Vec2),
}

impl Grid {
    pub fn get(&self, hex: hex::Hex) -> Option<&Entity> {
        self.storage.get(&hex)
    }

    pub fn set(&mut self, hex: hex::Hex, entity: Option<Entity>) -> Option<Entity> {
        match entity {
            Some(entity) => self.storage.insert(hex.clone(), entity),
            None => self.storage.remove(&hex),
        }
    }

    pub fn hex_to_world(&self, hex: hex::Hex) -> Vec2 {
        self.layout.hex_to_world(hex)
    }

    pub fn hex_to_world_y(&self, hex: hex::Hex, y: f32) -> Vec3 {
        let pos_2d = self.layout.hex_to_world(hex);
        Vec3::new(pos_2d.x, y, pos_2d.y)
    }

    pub fn world_to_hex(&self, pos: Vec3) -> hex::Hex {
        self.layout.world_to_hex(Vec2::new(pos.x, pos.z))
    }

    // TODO: this is not that efficient, but should be fine for now.
    pub fn update_bounds(&mut self) {
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        for (hex, _) in self.storage.iter() {
            let pos = self.layout.hex_to_world(hex.clone());
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
        }

        let (sx, sy) = self.hex_world_size();

        self.bounds = (
            Vec2::new(min_x - sx, max_x + sx),
            Vec2::new(min_y - sy, max_y + sx),
        );
    }

    pub fn ensure_centered(&mut self) {
        let (half_w, half_h) = (self.bounds.0.y / 2.0, self.bounds.1.y / 2.0);
        let (hw, hh) = self.hex_world_size();
        self.layout.origin = Vec2::new(-half_w + hw / 2., -half_h + hh / 2.);
        self.update_bounds();
    }

    pub fn hex_world_size(&self) -> (f32, f32) {
        self.layout.hex_world_size()
    }
}

fn generate_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<Grid>,
    grid_size: Res<GenerateGrid>,
) {
    for hex in hex::rectangle(grid_size.0, grid_size.1, grid.layout.orientation) {
        let world_pos = grid.hex_to_world_y(hex, 0.0);
        let entity = commands
            .spawn_bundle(BallBundle::new(
                world_pos,
                grid.layout.size.x,
                &mut meshes,
                &mut materials,
            ))
            .insert(hex)
            .insert(Name::new(
                format!("hex::Hex {:?}, {:?}", hex.q, hex.r).to_string(),
            ))
            .id();

        grid.set(hex, Some(entity));
    }

    grid.update_bounds();
    grid.ensure_centered();

    commands.remove_resource::<GenerateGrid>();
}

fn upkeep_hex_transforms(
    mut hexes: Query<(Entity, &mut Transform, &hex::Hex), Changed<hex::Hex>>,
    mut grid: ResMut<Grid>,
) {
    for (entity, mut transform, hex) in hexes.iter_mut() {
        let (x, z) = grid.hex_to_world(*hex).into();
        transform.translation.x = x;
        transform.translation.z = z;
        grid.set(*hex, Some(entity));
    }
}

fn display_grid_bounds(grid: Res<Grid>, mut lines: ResMut<DebugLines>) {
    const Z_LENGTH: f32 = 1000.;

    lines.line_colored(
        Vec3::new(grid.bounds.0.x, 0., Z_LENGTH),
        Vec3::new(grid.bounds.0.x, 0., -Z_LENGTH),
        0.,
        Color::GRAY,
    );
    lines.line_colored(
        Vec3::new(grid.bounds.0.y, 0., Z_LENGTH),
        Vec3::new(grid.bounds.0.y, 0., -Z_LENGTH),
        0.,
        Color::GRAY,
    );

    lines.line_colored(
        Vec3::new(0., 0., Z_LENGTH),
        Vec3::new(0., 0., -Z_LENGTH),
        0.,
        Color::GRAY,
    );

    lines.line_colored(
        Vec3::new(Z_LENGTH, 0., 0.),
        Vec3::new(-Z_LENGTH, 0., 0.),
        0.,
        Color::GRAY,
    );

    let xc: i32 = ((grid.bounds.0.y - grid.bounds.0.x) / (grid.layout.size.x * 2.)).ceil() as i32;
    let yc: i32 = ((grid.bounds.1.y - grid.bounds.1.x) / (grid.layout.size.y * 2.)).ceil() as i32;

    for hex in hex::rectangle(xc, yc + 40, grid.layout.orientation) {
        let corners = grid.layout.hex_corners(hex);
        for (i, c) in corners.iter().take(5).enumerate() {
            lines.line_colored(
                Vec3::new(c.x, 0.0, c.y),
                Vec3::new(corners[i + 1].x, 0.0, corners[i + 1].y),
                0.,
                Color::GRAY,
            );
        }

        lines.line_colored(
            Vec3::new(corners[0].x, 0.0, corners[0].y),
            Vec3::new(corners[5].x, 0.0, corners[5].y),
            0.,
            Color::GRAY,
        );
    }
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            layout: hex::Layout {
                orientation: hex::Orientation::Pointy,
                origin: Vec2::new(0.0, 0.0),
                size: Vec2::new(2.0, 2.0),
            },
            ..Default::default()
        });
        app.insert_resource(GenerateGrid(10, 10));
        app.add_system_set(SystemSet::on_enter(AppState::Next).with_system(generate_grid));
        app.add_system_set(SystemSet::on_update(AppState::Next).with_system(upkeep_hex_transforms));
        app.add_system_set(SystemSet::on_update(AppState::Next).with_system(display_grid_bounds));
    }
}
