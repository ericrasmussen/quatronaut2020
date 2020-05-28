/// An existential horror brought to life by the complexities of ECS.
use amethyst::{
    ecs::prelude::{Component, DenseVecStorage}
};

// since speed is shared, it's possible it could be a separate component.
// this would let us move it out of structs and eventually make it all
// configurable
pub struct Enemy {
    speed: f32,
}

impl Enemy {
    pub fn new(speed: f32) -> Enemy {
        Enemy {
            speed: speed,
        }
    }

    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
