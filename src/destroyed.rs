use crate::components::*;
use crate::CELL_SIZE;
use crate::GRID_SIZE;
use crate::WINDOW_SIZE;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

pub const DESTROYED_SPEED: f32 = CELL_SIZE * 8.;
pub const DESTROYED_ROTATION: f32 = 10.;
pub const SCRAPYARD_LOCATION: Vec2 = Vec2::splat(WINDOW_SIZE * 0.4);

pub fn destroyed_animation(
    mut commands: Commands,
    mut transforms: Query<(Entity, &mut Transform), With<Destroyed>>,
    time: Res<Time>,
) {
    for (entity, mut transform) in transforms.iter_mut() {
        // let destination = Vec3::from((SCRAPYARD_LOCATION, 0.));
        let destination =
            (transform.translation - Vec3::ZERO).normalize_or(Vec3::X) * WINDOW_SIZE * 0.6;

        let noise = Vec3::from((random_noise(CELL_SIZE), 0.));

        let direction = ((destination + noise) - transform.translation).normalize_or_zero();
        transform.translation += direction * DESTROYED_SPEED * time.delta_seconds();
        transform.rotate_local_z(-PI * DESTROYED_ROTATION * time.delta_seconds());

        // outside of the grid
        let margin = CELL_SIZE * 1.5;
        let playground_size = GRID_SIZE.as_vec2() * 0.5 * CELL_SIZE;

        let playground = Rect {
            min: -(playground_size + margin + random_noise(CELL_SIZE)),
            max: playground_size + margin + random_noise(CELL_SIZE),
        };
        if !playground.contains(transform.translation.xy()) {
            commands.entity(entity).remove::<Destroyed>();
        }
    }
}

pub fn random_noise(length: f32) -> Vec2 {
    let mut rng = rand::thread_rng();

    Vec2::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5).normalize_or(Vec2::X) * length
}
