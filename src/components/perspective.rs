//! This component lets us capture information about shaking the camera,
//! for use in certain level transitions (decided by `gameplay.rs`)
use amethyst::ecs::{storage::DenseVecStorage, Component};

use rand::{thread_rng, Rng};

use crate::resources::audio::SoundType;

/// Enum to tell us if we're still busy `Shaking` the camera or if we've
/// `Completed` the change in perspective.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PerspectiveStatus {
    Shaking,
    Completed,
}

use PerspectiveStatus::*;

/// This is intended to be a fairly high level API for controlling
/// transitions between small levels with the constrained play area.
/// In the game, it means between these levels the camera will shake
/// and make clunking sounds like something's wrong.
#[derive(Clone, Copy, Debug)]
pub struct Perspective {
    min_shake_seconds: f32,
    pub status: PerspectiveStatus,
    pub already_played_sound: bool,
    sound: SoundType,
}

impl Component for Perspective {
    type Storage = DenseVecStorage<Self>;
}

// amethyst requres `Default` for components used in systems, so
// this ensures it's empty (no shaking or sounds) in case it's ever
// created by amethyst and used in a system
impl Default for Perspective {
    fn default() -> Perspective {
        Perspective {
            min_shake_seconds: 0.0,
            status: Completed,
            already_played_sound: true,
            sound: SoundType::None,
        }
    }
}

impl Perspective {
    pub fn new(min_shake_seconds: f32, sound: SoundType) -> Perspective {
        Perspective {
            min_shake_seconds,
            status: Shaking,
            already_played_sound: false,
            sound,
        }
    }

    /// Get the sound associated with this perpsective.
    pub fn get_sound_type(self) -> SoundType {
        self.sound
    }

    /// Track whether or not we've played the sound yet.
    pub fn sound_already_played(self) -> bool {
        self.already_played_sound
    }

    /// Mark the sound as having already been played.
    pub fn played_sound(&mut self) {
        self.already_played_sound = true;
    }

    /// Compute the next value by which we'll modify the z axis of
    /// the camera transform (thus rotating our whole view).
    /// `time` should be delta seconds. See the systems module for
    /// real usage examples.
    pub fn next_z_rotation(&mut self, time: f32) -> Option<f32> {
        match self.status {
            Shaking => {
                // decrement time until we reach 0, then mark it as `Complete`
                self.min_shake_seconds -= time;
                if self.min_shake_seconds <= 0.0 {
                    self.status = Completed;
                }
                // this is a range in radians that will shake up the camera
                let mut rng = thread_rng();
                let next_rotation = rng.gen_range(-0.5, 0.5);
                Some(next_rotation * time)
            },
            _ => None,
        }
    }
}
