use crate::components::*;
use crate::movements::position_translation;
use bevy::prelude::*;

use crate::CELL_SIZE;
use crate::GRID_SIZE;

pub fn draw_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            GRID_SIZE,
            Vec2::new(CELL_SIZE, CELL_SIZE),
            LinearRgba::gray(0.05),
        )
        .outer_edges();
}

pub fn draw_targets(mut gizmos: Gizmos, journeys: Query<&Journey, With<Player>>) {
    for journey in journeys.iter() {
        gizmos.rounded_rect_2d(
            position_translation(&journey.start_pos) + CELL_SIZE / 2.,
            0.,
            Vec2::splat(CELL_SIZE),
            journey.color,
        );
        gizmos.circle_2d(
            position_translation(&journey.target_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * 1.1,
            journey.color,
        );
    }
}

pub fn draw_paths(mut gizmos: Gizmos, journeys: Query<&Journey>) {
    for journey in journeys.iter() {
        for pos in journey.path.iter() {
            gizmos.rect_2d(
                position_translation(pos) + CELL_SIZE / 2.,
                0.,
                Vec2::splat(CELL_SIZE * journey.scale),
                journey.color.with_alpha(0.05),
            );
        }
    }
}
