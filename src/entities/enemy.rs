/// An existential horror brought to life by the complexities of ECS.
use amethyst::ecs::prelude::{Component, DenseVecStorage};

//use log::info;

// since speed is shared, it's possible it could be a separate component.
// this would let us move it out of structs and eventually make it all
// configurable
// the velocity is used for our tracking system, which isn't able to update transforms
pub struct Enemy {
    speed: f32,
    // TODO: write a function to provide these
    pub velocity_x: f32,
    pub velocity_y: f32,
}

impl Enemy {
    pub fn new(speed: f32) -> Enemy {
        Enemy {
            speed,
            velocity_x: 0.0,
            velocity_y: 0.0,
        }
    }

    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    // probably doesn't belong here but since only enemies need this for now,
    // here's a function to compute how to move towards another transform
    // based on speed
    pub fn move_towards(&mut self, target_x: f32, target_y: f32, current_x: f32, current_y: f32) {
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let angle = dy.atan2(dx);

        self.velocity_x = &self.get_speed() * angle.cos();
        self.velocity_y = &self.get_speed() * angle.sin();
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
