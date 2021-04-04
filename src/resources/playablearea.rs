//! This struct is used by systems that update transforms by making
//! it easy to restrict movement to a specified playable area. This
//! is intended for use by `gameplay.rs` and should be based on the
//! screen dimensions.
use amethyst::ecs::{storage::DenseVecStorage, Component};

/// Enum for easy matching on what we're trying to restrict.
pub enum ClampDimension {
    ClampX,
    ClampY,
}

/// This is the main struct to be used by other systems and states, which
/// will let them check if something is heading out of bounds.
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
    // given the computed width, height, and constraint, create a new playarea.
    // if the background dimensions change then the hardcoded % values here will need
    // to be adjusted. note that in true developer "works-for-me" fashion,
    // this only currently supports normal displays and retina displays (which
    // is determined by the hidpi factor the amethyst `ScreenDimensions`).
    // this is likely a bad assumption because not all hidpi monitors will have
    // the same number of pixels as a retina display, but currently it's all I can
    // test on
    pub fn new(width: f32, height: f32, constrain: bool, hidpi: bool) -> PlayableArea {
        // these percentages were calculated manually based on the position of
        // the black rectangle in the smaller, unbroken background image, then
        // adjusted as needed for the retina display version
        if constrain {
            let (x1, x2, y1, y2) = if hidpi {
                (0.33, 0.67, 0.22, 0.78)
            } else {
                (0.25, 0.75, 0.05, 0.95)
            };
            PlayableArea {
                min_x: width * x1,  // normal: 0.25, retina: 0.33,
                max_x: width * x2,  // normal: 0.75, retina: 0.67,
                min_y: height * y1, // normal: 0.05, retina: 0.22,
                max_y: height * y2, // normal: 0.95, retina: 0.78,
            }
        } else {
            // there needs to be some buffer here so everything is visible inside the camera.
            // otherwise some things can be rendered in the center of the border and be partially
            // offscreen
            let (x1, x2, y1, y2) = if hidpi {
                (0.17, 0.83, 0.20, 0.80)
            } else {
                (0.02, 0.98, 0.045, 0.955)
            };
            PlayableArea {
                min_x: width * x1,  // normal: 0.02, retina: 0.17,
                max_x: width * x2,  // normal: 0.98, retina: 0.83,
                min_y: height * y1, // normal: 0.045, retina: 0.20,
                max_y: height * y2, // normal: 0.955, retina: 0.80,
            }
        }
    }

    /// Computes coordinates in the playable area based on a relative percentage
    /// (e.g. a level config that specifies an enemy should be 25% of the way along
    /// the x dimension, and 30% of the way along the y dimension). The supplied
    /// arguments should be between 0.0 and 1.0, inclusive.
    pub fn relative_coordinates(&self, x_percentage: &f32, y_percentage: &f32) -> (f32, f32) {
        let x_diff = self.max_x - self.min_x;
        let y_diff = self.max_y - self.min_y;
        let x_pos = x_percentage * x_diff + self.min_x;
        let y_pos = y_percentage * y_diff + self.min_y;
        (x_pos, y_pos)
    }

    /// Can be used to see if some given x, y has traveled outside the playing area.
    pub fn out_of_bounds(&self, x: f32, y: f32) -> bool {
        x < self.min_x || x > self.max_x || y < self.min_y || y > self.max_y
    }

    /// API for clamping (restricting) the player so that when they try to
    /// travel beyond some min or max x value on the horizontal access, they
    /// can't move further.
    pub fn clamp_x(&self, n: f32) -> f32 {
        self.clamp(n, ClampDimension::ClampX)
    }

    /// API for clamping (restricting) the player so that when they try to
    /// travel beyond some min or max y value on the horizontal access, they
    /// can't move further.
    pub fn clamp_y(&self, n: f32) -> f32 {
        self.clamp(n, ClampDimension::ClampY)
    }

    /// The way `clamp` ultimately works is it will see if some value
    /// has gone below or above the configured min/max of the play area
    /// (on either the horizontal or vertical axis, as determined by the
    /// `ClampDimension`). If the value does go beyond one of those, we
    /// return the `min` or `max`, e.g. if the player is at 0 and tries to
    /// go to -1, they just end up staying at 0. If the value is not
    /// out of bounds, it gets returned as-is (e.g. if the player is at 0
    /// and tries to go to 1, they can).
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
