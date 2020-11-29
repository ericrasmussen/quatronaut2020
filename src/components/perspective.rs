/// This is a fun little module for describing how
/// we manipulate the camera during our transition to
/// the damaged background/wide-screen mode.
use amethyst::{
    core::math::Vector3,
    ecs::{storage::DenseVecStorage, Component},
};

use rand::{thread_rng, Rng};

use log::info;

#[derive(Clone, Copy, Debug)]
pub struct Perspective {
    scale_factor: f32,
    scale_min: f32,
    completed: bool,
    reversing: bool,
}

impl Component for Perspective {
    type Storage = DenseVecStorage<Self>;
}

impl Default for Perspective {
    fn default() -> Perspective {
        Perspective {
            scale_factor: 0.0,
            scale_min: 0.0,
            completed: false,
            reversing: false,
        }
    }
}

impl Perspective {
    pub fn new(scale_factor: f32, scale_min: f32) -> Perspective {
        Perspective {
            scale_factor,
            scale_min,
            completed: false,
            reversing: false,
        }
    }

    pub fn is_reversing(self) -> bool {
        self.reversing
    }

    pub fn is_completed(self) -> bool {
        self.completed
    }

    // compute the next value by which we'll modify the z axis of
    // the camera transform (thus rotating our whole view)
    // note that the timing and presentation all depends on scaling,
    // so this function does not modify self.completed or self.reversing
    pub fn next_z_rotation(&mut self, time: f32) -> Option<f32> {
        if self.completed || self.reversing {
            None
        } else {
            // this is a range in radians that will shake up the camera
            let mut rng = thread_rng();
            let next_rotation = rng.gen_range(-0.5, 0.5);
            Some(next_rotation * time)
        }
    }

    // compute the next value by which to scale the camera. increasing
    // the value creates a zooming out effect
    pub fn next_scale(&mut self, current_scale: f32, time: f32) -> Option<Vector3<f32>> {
        // we're all done!
        if self.completed {
            None
        }
        // we're going back to normal scale and...
        else if self.reversing {
            let new_scale = current_scale + (self.scale_factor * time);
            // we've gone too far. reset and stop!
            if new_scale >= 1.0 {
                self.completed = true;
                self.reversing = false;
                Some(Vector3::from_element(1.0))
            }
            // otherwise keep going
            else {
                Some(Vector3::new(new_scale, new_scale, new_scale))
            }
        }
        // we're all done zooming in; time to go back
        else if current_scale <= self.scale_min {
            info!("going back!!!! {:?}", current_scale);
            self.reversing = true;
            None
        }
        // we're not done zooming in yet
        else {
            let new_scale = current_scale - (self.scale_factor * time);
            Some(Vector3::new(new_scale, new_scale, new_scale))
        }
    }
}
