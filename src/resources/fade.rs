/// This state can be pushed on top of `GameplayState`
/// and popped as needed. For now its main purpose is having
/// a kind of cutscene/level complete transition so that
/// progressing to the next level isn't so jarring.
use amethyst::ecs::{storage::DenseVecStorage, Component};

#[derive(Clone, Debug, PartialEq)]
pub enum Fade {
    Darken,
    Lighten,
    Done,
}

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

    pub fn fade_completed(&self) -> bool {
        self.fade_direction == Fade::Done
    }

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

    pub fn is_darkened(&self) -> bool {
        self.fade_direction == Fade::Darken && self.alpha >= 1.0
    }

    pub fn is_lightened(&self) -> bool {
        self.fade_direction == Fade::Lighten && self.alpha <= 0.0
    }
}

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
    pub fn update(&mut self, fader: Fader) {
        if fader.fade_completed() {
            self.completed = true;
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn clear(&mut self) {
        self.completed = false;
    }
}
