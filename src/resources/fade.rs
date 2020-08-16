/// This state can be pushed on top of `GameplayState`
/// and popped as needed. For now its main purpose is having
/// a kind of cutscene/level complete transition so that
/// progressing to the next level isn't so jarring.
use amethyst::ecs::{storage::DenseVecStorage, Component};

#[derive(Clone, Debug, PartialEq)]
pub enum Fade {
    Darken,
    Lighten,
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
        };
        Fader {
            fade_speed,
            fade_direction,
            alpha,
        }
    }

    pub fn next_alpha_change(&mut self, time_delta: f32) -> f32 {
        let change_amt = self.fade_speed * time_delta;
        match self.fade_direction {
            Fade::Darken => self.alpha += change_amt,
            Fade::Lighten => self.alpha -= change_amt,
        }

        // TESTING!!! hope you aren't reading this on github
        if self.is_darkened() || self.is_lightened() {
            self.switch();
        }

        self.alpha
    }

    pub fn is_darkened(&self) -> bool {
        self.fade_direction == Fade::Darken && self.alpha >= 1.0
    }

    pub fn is_lightened(&self) -> bool {
        self.fade_direction == Fade::Lighten && self.alpha <= 0.0
    }

    pub fn switch(&mut self) {
        match self.fade_direction {
            Fade::Darken => {
                self.fade_direction = Fade::Lighten;
                self.alpha = 1.0;
            },
            Fade::Lighten => {
                self.fade_direction = Fade::Darken;
                self.alpha = 0.0;
            },
        }
    }
}
