/// This module is for
use amethyst::ecs::prelude::{Component, DenseVecStorage};

use crate::resources::direction::Direction;

//use log::info;

// An individual glass shard with its own direction and speed
// TODO: maybe a delay here?
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
