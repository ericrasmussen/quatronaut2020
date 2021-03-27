/// This is a fun little module for describing how
/// we manipulate the camera during our transition to
/// the damaged background/wide-screen mode.
use amethyst::{
    core::math::Vector3,
    ecs::{storage::DenseVecStorage, Component},
};

use rand::{thread_rng, Rng};

//use log::info;

use crate::resources::audio::SoundType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PerspectiveStatus {
    Zooming,
    Reversing,
    // TODO: add in a paused status here so we can delay for a bit before
    // the dramatic zoom out.
    //    Paused,
    Completed,
}

use PerspectiveStatus::*;

#[derive(Clone, Copy, Debug)]
pub struct Perspective {
    scale_factor: f32,
    scale_min: f32,
    pub status: PerspectiveStatus,
    already_played_sound: bool,
    sound: SoundType,
}

impl Component for Perspective {
    type Storage = DenseVecStorage<Self>;
}

impl Default for Perspective {
    fn default() -> Perspective {
        Perspective {
            scale_factor: 0.0,
            scale_min: 0.0,
            status: Zooming,
            already_played_sound: false,
            sound: SoundType::ShortTransition,
        }
    }
}

impl Perspective {
    pub fn new(scale_factor: f32, scale_min: f32, sound: SoundType) -> Perspective {
        Perspective {
            scale_factor,
            scale_min,
            status: Zooming,
            already_played_sound: false,
            sound,
        }
    }

    pub fn get_sound_type(self) -> SoundType {
        self.sound
    }

    pub fn sound_already_played(self) -> bool {
        self.already_played_sound
    }

    pub fn played_sound(&mut self) {
        self.already_played_sound = true;
    }

    // compute the next value by which we'll modify the z axis of
    // the camera transform (thus rotating our whole view)
    // note that the timing and presentation all depends on scaling,
    // so this function does not modify self.completed or self.reversing
    pub fn next_z_rotation(&mut self, time: f32) -> Option<f32> {
        match self.status {
            Zooming => {
                // this is a range in radians that will shake up the camera
                let mut rng = thread_rng();
                let next_rotation = rng.gen_range(-0.5, 0.5);
                Some(next_rotation * time)
            },
            _ => None,
        }
    }

    // compute the next value by which to scale the camera. increasing
    // the value creates a zooming out effect
    pub fn next_scale(&mut self, current_scale: f32, time: f32) -> Option<Vector3<f32>> {
        match self.status {
            // all done!
            Completed => None,
            // going back to normal scale
            Reversing => {
                let new_scale = current_scale + (self.scale_factor * time);
                // we've gone too far. reset and stop!
                if new_scale >= 1.0 {
                    self.status = Completed;
                    Some(Vector3::from_element(1.0))
                // more reversing to do still
                } else {
                    Some(Vector3::new(new_scale, new_scale, new_scale))
                }
            },
            // still zoomin'
            Zooming => {
                // if we've zoomed past our threshold, start reversing
                if current_scale <= self.scale_min {
                    self.status = Reversing;
                    None
                // otherwise keep going
                } else {
                    let new_scale = current_scale - (self.scale_factor * time);
                    Some(Vector3::new(new_scale, new_scale, new_scale))
                }
            },
        }
    }
}
