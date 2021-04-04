//! The `Cutscene` component is used to describe how we can manipulate the
//! camera during our transition to thedamaged background art/wide-screen mode.
//! It's expected that camera systems will use this component to get the next
//! camera scale, and states (namely `transition.rs`) can check if we're in
//! the `Spawning` phase (meaning it's time to spawn glass shards so it looks
//! like the background is starting to break) or `Completed` (time to
//! transition back to the game).
use amethyst::{
    core::math::Vector3,
    ecs::{storage::DenseVecStorage, Component},
};

use crate::resources::audio::SoundType;

/// Enum used to check in on the status of the cutscene. It's
/// up to callers to move on after this is `Completed`, so be careful
/// not to accidentally let a cutscene run forever.
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
    pub fn new(zoom_in_duration: f32, zoom_in_scale: f32, spawn_duration: f32, zoom_out_duration: f32) -> Cutscene {
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

    /// Check which configured `SoundType` should be used.
    pub fn get_sound_type(self) -> SoundType {
        self.sound_type
    }

    /// Lets systems check periodically to see if the sound
    /// was already played once, in which case it shouldn't be played
    /// again.
    pub fn sound_already_played(self) -> bool {
        self.already_played_sound
    }

    /// Lets callers mark this as having already played a sound.
    pub fn played_sound(&mut self) {
        self.already_played_sound = true;
    }

    /// Computes the next value by which to scale the camera. increasing
    /// the value creates a zooming out effect. this is called repeatedly
    /// by systems until it returns None
    pub fn next_scale(&mut self, current_scale: f32, time: f32) -> Option<Vector3<f32>> {
        match self.status {
            // all done!
            Completed => None,
            // going back to normal scale
            Reversing => {
                // the `scale_factor` (duplicated in `Zooming`) calculates how far
                // away we are from the desired scale, then divides it by how long this
                // operation should take. when multiplied against delta time (the time
                // elapsed since the last frame), this let's us scale in incremental
                // amounts that will reach the desired scale at the given duration
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_soundtype() {
        let cutscene = Cutscene::new(0.5, 0.4, 5.0, 2.0);
        assert_eq!(cutscene.get_sound_type(), SoundType::GlassTransition);
    }

    #[test]
    fn test_sound_flag() {
        let mut cutscene = Cutscene::new(0.5, 0.4, 5.0, 2.0);
        assert_eq!(cutscene.sound_already_played(), false);
        cutscene.played_sound();
        assert_eq!(cutscene.sound_already_played(), true);
    }

    #[test]
    fn test_completed() {
        let mut cutscene = Cutscene::new(0.5, 0.4, 5.0, 2.0);
        cutscene.status = Completed;
        assert_eq!(cutscene.next_scale(1.0, 1.0), None);
    }

    #[test]
    fn test_reversing() {
        // mostly testing that it returns something before it reaches the desired
        // scale, then it returns 1.0 (since we want everything back to regular
        // 1.0 scale) and marks the status as `Complete`
        let mut cutscene = Cutscene::new(0.5, 0.4, 5.0, 2.0);
        cutscene.status = Reversing;
        assert_eq!(cutscene.next_scale(0.3, 0.5), Some(Vector3::from_element(0.45000002)));
        assert_eq!(cutscene.next_scale(1.0, 1.0), Some(Vector3::from_element(1.0)));
        assert_eq!(cutscene.status, Completed);
    }
}
