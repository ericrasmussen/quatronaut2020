/// This struct is used by systems that update transforms by making
/// it easy to restrict movement to a specified playable area. This
/// is intended for use by `gameplay.rs` and should be based on the
/// screen dimensions.
use amethyst::ecs::{storage::DenseVecStorage, Component};

pub enum ClampDimension {
    ClampX,
    ClampY,
}

#[derive(Clone, Debug)]
pub struct PlayableArea {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Default for PlayableArea {
    fn default() -> Self {
        PlayableArea {
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        }
    }
}

impl PlayableArea {
    // given the computed width, height, and constraint, create a new playarea
    // if the background dimensions then the hardcoded values here will need
    // to be adjusted
    pub fn new(width: f32, height: f32, constrain: bool) -> PlayableArea {
        // these percentages were calculated manually based on the position of
        // the black rectangle in the smaller, unbroken background image
        if constrain {
            PlayableArea {
                min_x: width * 0.33,
                max_x: width * 0.67,
                min_y: height * 0.22,
                max_y: height * 0.78,
            }
        } else {
            // there needs to be some buffer here so everything is visible inside the camera.
            // otherwise some things can be rendered in the center of the border and be partially
            // offscreen
            PlayableArea {
                min_x: width * 0.17,
                max_x: width * 0.83,
                min_y: height * 0.20,
                max_y: height * 0.80,
            }
        }
    }

    // computes coordinates in the playable area based on a relative percentage
    // (e.g. a level config that specifies an enemy should be 25% of the way along
    // the x dimension, and 30% of the way along the y dimension). The supplied
    // arguments should be between 0.0 and 1.0, inclusive
    pub fn relative_coordinates(&self, x_percentage: &f32, y_percentage: &f32) -> (f32, f32) {
        let x_diff = self.max_x - self.min_x;
        let y_diff = self.max_y - self.min_y;
        let x_pos = x_percentage * x_diff + self.min_x;
        let y_pos = y_percentage * y_diff + self.min_y;
        (x_pos, y_pos)
    }

    // can be used to see if some given x, y has traveled outside the playing area.
    pub fn out_of_bounds(&self, x: f32, y: f32) -> bool {
        x < self.min_x || x > self.max_x || y < self.min_y || y > self.max_y
    }

    pub fn clamp_x(&self, n: f32) -> f32 {
        self.clamp(n, ClampDimension::ClampX)
    }

    pub fn clamp_y(&self, n: f32) -> f32 {
        self.clamp(n, ClampDimension::ClampY)
    }

    fn clamp(&self, n: f32, clamp_dimension: ClampDimension) -> f32 {
        let (min, max) = match clamp_dimension {
            ClampDimension::ClampX => (self.min_x, self.max_x),
            ClampDimension::ClampY => (self.min_y, self.max_y),
        };

        if n < min {
            min
        } else if n > max {
            max
        } else {
            n
        }
    }
}

impl Component for PlayableArea {
    type Storage = DenseVecStorage<Self>;
}
