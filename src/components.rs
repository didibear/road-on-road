use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct GameObject;

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub struct Position(pub IVec2);

#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct Transition {
    pub start: Position,
    pub end: Position,
    pub current: Vec2,
}

impl Transition {
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            current: start.0.as_vec2(),
        }
    }
}

#[derive(Debug, Component)]
pub struct Player;

#[derive(Debug, Component)]
pub struct GameFinishedPlayer;

#[derive(Debug, Component)]
pub struct Automated;

pub type WithPlayerOrAutomated = Or<(With<Automated>, With<Player>)>;

#[derive(Debug, Component)]
pub struct Destroyed;

#[derive(Debug, Component)]
pub struct Journey {
    pub start_pos: Position,
    pub target_pos: Position,
    pub path: Vec<Position>,
    pub bot_index: i32,
    // display
    pub color: Color,
    pub scale: f32,
}

#[derive(Event)]
pub struct JourneyFinished;
