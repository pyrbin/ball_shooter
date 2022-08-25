use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

pub const FOREVER: f32 = std::f32::MAX;

pub trait DebugLinesExt {
    fn circle(&mut self, origin: Vec3, rot: Quat, radius: f32, duration: f32, color: Color);
}

impl DebugLinesExt for DebugLines {
    fn circle(&mut self, origin: Vec3, rot: Quat, radius: f32, duration: f32, color: Color) {
        add_circle(self, origin, rot, radius, duration, color);
    }
}

fn add_circle(
    lines: &mut DebugLines,
    origin: Vec3,
    rot: Quat,
    radius: f32,
    duration: f32,
    color: Color,
) {
    let x_rotate = Quat::from_rotation_x(PI);
    add_semicircle(lines, origin, rot, radius, duration, color);
    add_semicircle(lines, origin, rot * x_rotate, radius, duration, color);
}

fn add_semicircle(
    lines: &mut DebugLines,
    origin: Vec3,
    rot: Quat,
    radius: f32,
    duration: f32,
    color: Color,
) {
    let x_rotate = Quat::from_rotation_y(PI);
    add_quartercircle(lines, origin, rot, radius, duration, color);
    add_quartercircle(lines, origin, rot * x_rotate, radius, duration, color);
}

fn add_quartercircle(
    lines: &mut DebugLines,
    origin: Vec3,
    rot: Quat,
    radius: f32,
    duration: f32,
    color: Color,
) {
    let quarter_circle_segments = 4;
    let angle = FRAC_PI_2 / quarter_circle_segments as f32;
    let mut current_point = rot.mul_vec3(Vec3::X * radius);
    let direction = Quat::from_axis_angle(rot.mul_vec3(Vec3::Y), angle);
    for _ in 0..quarter_circle_segments {
        let next_point = direction.mul_vec3(current_point);
        lines.line_colored(origin + current_point, origin + next_point, duration, color);
        current_point = next_point;
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::with_depth_test(true));
    }
}
