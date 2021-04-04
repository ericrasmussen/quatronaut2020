//! The `Glass` component represents an individual glass shard
//! that gets placed on the screen when we "break" the small
//! background and reveal the larger, broken background (play the
//! game to see it in action -- just press 'g' during a level
//! for invulnerability).
use amethyst::ecs::prelude::{Component, DenseVecStorage};

use crate::resources::direction::Direction;

// An individual glass shard with its own direction and speed
#[derive(Debug)]
pub struct Glass {
    pub direction: Direction,
    pub speed: f32,
}

impl Glass {
    pub fn new(direction: Direction, speed: f32) -> Glass {
        Glass { direction, speed }
    }
}

impl Component for Glass {
    type Storage = DenseVecStorage<Self>;
}
