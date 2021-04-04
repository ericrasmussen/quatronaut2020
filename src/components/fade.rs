//! This component provides an API for controlling fade to black transitions.
use amethyst::ecs::{storage::DenseVecStorage, Component};

/// This enum lets us track the status of the current fade transition.
#[derive(Clone, Debug, PartialEq)]
pub enum Fade {
    Darken,
    Lighten,
    Done,
}

/// The `Fader` is given a speed, a direction (Darken/Lighten/Done),
/// and an alpha level (0.0 is transparent, 1.0 is solid black).
#[derive(Clone, Debug)]
pub struct Fader {
    fade_speed: f32,
    fade_direction: Fade,
    alpha: f32,
}

impl Component for Fader {
    type Storage = DenseVecStorage<Self>;
}

impl Fader {
    pub fn new(fade_speed: f32, fade_direction: Fade) -> Fader {
        let alpha = match fade_direction {
            Fade::Darken => 0.0,
            Fade::Lighten => 1.0,
            // no one should create a new instance that's already
            // done fading, but if so, we don't want to modify the alpha
            Fade::Done => 0.0,
        };
        Fader {
            fade_speed,
            fade_direction,
            alpha,
        }
    }

    /// Whether or not we're all done fading.
    pub fn fade_completed(&self) -> bool {
        self.fade_direction == Fade::Done
    }

    /// Compute the next alpha change based on the time since the last
    /// frame and how fast we want to fade.
    pub fn next_alpha_change(&mut self, time_delta: f32) -> f32 {
        let change_amt = self.fade_speed * time_delta;
        match self.fade_direction {
            Fade::Darken => self.alpha += change_amt,
            Fade::Lighten => self.alpha -= change_amt,
            Fade::Done => {},
        }

        if self.is_darkened() {
            self.fade_direction = Fade::Lighten;
        } else if self.is_lightened() {
            self.fade_direction = Fade::Done;
        }

        self.alpha
    }

    /// Check if we're all done covering the screen.
    pub fn is_darkened(&self) -> bool {
        self.fade_direction == Fade::Darken && self.alpha >= 1.0
    }

    /// Check if we're all done making the fader transparent.
    pub fn is_lightened(&self) -> bool {
        self.fade_direction == Fade::Lighten && self.alpha <= 0.0
    }
}

/// This component can be stored in the amethyst `world` to keep track
/// of whether or not we're done fading. This is somewhat inconsistent
/// with other APIs in this codebase, where similar flags are stored at
/// the state level or passed around in config structs.
/// Variety! The spice of life!
pub struct FadeStatus {
    completed: bool,
}

impl Component for FadeStatus {
    type Storage = DenseVecStorage<Self>;
}

impl Default for FadeStatus {
    fn default() -> FadeStatus {
        FadeStatus { completed: false }
    }
}

impl FadeStatus {
    /// Update the status based on whether or not the `fader` is done.
    pub fn update(&mut self, fader: Fader) {
        if fader.fade_completed() {
            self.completed = true;
        }
    }

    /// Check if we're all done.
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Clear the status (useful if we want to change fade direction
    /// or otherwise keep going).
    pub fn clear(&mut self) {
        self.completed = false;
    }
}
