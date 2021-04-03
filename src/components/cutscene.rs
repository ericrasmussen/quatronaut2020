/// This is a fun little module for describing how
/// we manipulate the camera during our transition to
/// the damaged background/wide-screen mode.
/// By fun, I mostly mean overly complicated. The short and long
/// transitions should really be separated out. Changing one or the other
/// is likely to break something.
use amethyst::{
    core::math::Vector3,
    ecs::{storage::DenseVecStorage, Component},
};

//use log::info;

use crate::resources::audio::SoundType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CutsceneStatus {
    Zooming,
    Reversing,
    Spawning,
    Completed,
}

use CutsceneStatus::*;

/// Zooming works by decreasing the scale of the camera's transform
/// (as it grows smaller in scale, everything in the viewport appears
/// bigger).
/// This struct lets us decide how long a cutscene should spend zooming
/// and reversing, the max scale of the camera transform (i.e. how far
/// to zoom in), the status of the cutscene (used by `transition.rs`),
/// the sound to play, if the sound has been played yet, and how long
/// to spend in the spawn phase where we generate glass shards on the
/// screen.
/// Phew, that's a lot.
#[derive(Clone, Copy, Debug)]
pub struct Cutscene {
    pub status: CutsceneStatus,
    zoom_in_duration: f32,
    zoom_in_scale: f32,
    spawn_duration: f32,
    zoom_out_duration: f32,
    sound_type: SoundType,
    pub already_played_sound: bool,
}

impl Component for Cutscene {
    type Storage = DenseVecStorage<Self>;
}

/// Components need a `Default` implementation. It shouldn't be
/// used anywhere, but the default values amount to no cutscene just in case.
impl Default for Cutscene {
    fn default() -> Cutscene {
        Cutscene {
            status: Completed,
            zoom_in_duration: 0.0,
            zoom_in_scale: 1.0, // don't decrease size at all
            spawn_duration: 0.0,
            zoom_out_duration: 0.0,
            sound_type: SoundType::GlassTransition,
            already_played_sound: true,
        }
    }
}

impl Cutscene {
    pub fn new(
        zoom_in_duration: f32,
        zoom_in_scale: f32,
        spawn_duration: f32,
        zoom_out_duration: f32,
    ) -> Cutscene {
        Cutscene {
            status: Zooming, // zooming in starts the cutscene
            zoom_in_duration,
            zoom_in_scale,
            spawn_duration,
            zoom_out_duration,
            sound_type: SoundType::GlassTransition,
            already_played_sound: false,
        }
    }

    pub fn get_sound_type(self) -> SoundType {
        self.sound_type
    }

    pub fn sound_already_played(self) -> bool {
        self.already_played_sound
    }

    pub fn played_sound(&mut self) {
        self.already_played_sound = true;
    }

    // compute the next value by which to scale the camera. increasing
    // the value creates a zooming out effect. this is called repeatedly
    // by systems until it returns None
    pub fn next_scale(&mut self, current_scale: f32, time: f32) -> Option<Vector3<f32>> {
        match self.status {
            // all done!
            Completed => {
                None
            },
            // going back to normal scale
            Reversing => {
                let scale_factor = (1.0 - self.zoom_in_scale) / self.zoom_out_duration;
                let new_scale = current_scale + (scale_factor * time);
                // we've gone too far. reset and stop!
                if new_scale >= 1.0 {
                    self.status = Completed;
                    Some(Vector3::from_element(1.0))
                // more reversing to do still
                } else {
                    Some(Vector3::new(new_scale, new_scale, new_scale))
                }
            },
            // start reversing when enough time has elapsed, otherwise keep
            // returning the current scale (effectively pausing the camera
            // changes). another system will use this opportunity to spawn
            // glass shards
            Spawning => {
                self.spawn_duration -= time;
                if self.spawn_duration <= 0.0 {
                    self.status = Reversing;
                }
                Some(Vector3::new(current_scale, current_scale, current_scale))
            },
            // still zoomin'
            Zooming => {
                // if we've zoomed past our threshold, start the spawn phase
                // of the cutscene
                // sufficiently, pause before reversing
                if current_scale <= self.zoom_in_scale {
                    self.status = Spawning;
                    None
                // otherwise keep going
                } else {
                    let scale_factor = (1.0 - self.zoom_in_scale) / self.zoom_in_duration;
                    let new_scale = current_scale - (scale_factor * time);
                    Some(Vector3::new(new_scale, new_scale, new_scale))
                }
            },
        }
    }
}
