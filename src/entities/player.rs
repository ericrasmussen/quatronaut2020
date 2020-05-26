/// The `Player` in our game is a robot, presumably named Benitron 3000.
/// The robot has its own movement speed and fire delay, the latter of which
/// is used to prevent the laser firing as fast as possible based on the
/// current framerate.
use amethyst::{
    ecs::prelude::{Component, DenseVecStorage}
};

pub struct Player {
    speed: f32,
    // time to delay laser shots in seconds
    fire_delay: f32,
    seconds_since_firing: f32,
}

impl Player {
    pub fn new(speed: f32, fire_delay: f32) -> Player {
        Player {
            speed: speed,
            fire_delay: fire_delay,
            // we don't want a delay for the very first laser blast
            seconds_since_firing: fire_delay,
        }
    }

    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    // checks if we've had enough time elapse since the last laser
    // and resets the timer. this is possibly a surprising API for a
    // `bool` check, but it also ensures we don't rely on calling code
    // to manage the timer.
    pub fn can_fire(&mut self, time: f32) -> bool {
        if self.seconds_since_firing >= self.fire_delay {
            self.seconds_since_firing = 0.0;
            true
        }
        else {
            self.seconds_since_firing += time;
            false
        }
    }


}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}
