//! In a top-down 2d game, there are times where components may
//! rotate to any point, say rotating some number of degrees to
//! face the player. This is not the module for those times. This
//! module is for the possible directions you might choose when
//! firing a laser or moving the player, such as moving right,
//! moving right and up, moving left and down, etc.
use amethyst::core::math::Vector3;
use rand::distributions::{Distribution, Standard};
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ManualDirection {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub radians: f32,
}

impl ManualDirection {
    pub fn new(player_x: f32, player_y: f32, player_z: f32, mouse_x: f32, mouse_y: f32) -> ManualDirection {
        let (velocity_x, velocity_y) = ManualDirection::get_velocities(player_x, player_y, mouse_x, mouse_y);
        let mouse_vec = Vector3::new(mouse_x, mouse_y, player_z);
        let player_vec = Vector3::new(player_x, player_y, player_z);
        let radians = ManualDirection::manual_radians(mouse_vec, player_vec);
        ManualDirection {
            velocity_x,
            velocity_y,
            radians,
        }
    }

    fn manual_radians(mouse_vec: Vector3<f32>, player_vec: Vector3<f32>) -> f32 {
        let dir = mouse_vec - player_vec;
        let angle = dir.y.atan2(dir.x);
        angle - (90.0 * PI / 180.0)
    }

    fn get_velocities(player_x: f32, player_y: f32, mouse_x: f32, mouse_y: f32) -> (f32, f32) {
        let dx = mouse_x - player_x;
        let dy = mouse_y - player_y;
        let angle = dy.atan2(dx);

        (angle.cos(), angle.sin())
    }
}

/// The main `Direction` enum for capturing the direction
/// of the player, lasers, and glass shards.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum Direction {
    Left,
    Up,
    LeftUp,
    LeftDown,
    Right,
    Down,
    RightUp,
    RightDown,
    Mouse(ManualDirection),
}

use Direction::*;

/// Allows callers to randomly generate directions for spawning.
impl Distribution<Direction> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        // randomly chooses a number from 1-8
        // (`gen_range` is inclusive low/exclusvie high)
        let n: u32 = rng.gen_range(1 .. 9);
        match n {
            1 => Left,
            2 => Up,
            3 => LeftUp,
            4 => LeftDown,
            5 => Right,
            6 => Down,
            7 => RightUp,
            _ => RightDown,
        }
    }
}

// the sprites in this game default to facing up.
// if that changes, so should this
impl Default for Direction {
    fn default() -> Direction {
        Up
    }
}

impl Direction {
    /// Inputs from wasd and the arrow keys will be mapped to values,
    /// where -1.0 is left ("negative"), 0.0 is neutral, and 1.0 is
    /// right ("positive"). Analog sticks or other controllers may
    /// provide varying degrees of positive/negative, so this may need
    /// adjustment if it changes from keyboard input.
    pub fn horizontal(x: f32) -> Option<Direction> {
        if x < 0.0 {
            Some(Left)
        } else if x > 0.0 {
            Some(Right)
        } else {
            None
        }
    }

    /// Same as `horizontal` except -1.0 is down and 1.0 is up.
    pub fn vertical(y: f32) -> Option<Direction> {
        if y < 0.0 {
            Some(Down)
        } else if y > 0.0 {
            Some(Up)
        } else {
            None
        }
    }

    /// This approach is a little messy. we're assuming it's only ever used
    /// to combine horizontal with vertical. if it's not used that way then it's
    /// meaningless, because we can't have RightRight, or RightLeft, and so on.
    pub fn combine(self, other: &Option<Direction>) -> Direction {
        match (self, other) {
            (Right, Some(Up)) => RightUp,
            (Right, Some(Down)) => RightDown,
            (Left, Some(Up)) => LeftUp,
            (Left, Some(Down)) => LeftDown,
            (x, _) => x,
        }
    }

    /// Given some horizontal `x` and vertical `y` values in the
    /// range (-1., 0., 1.), this attempts to create a `Direction`.
    /// If the player isn't moving at all it'll evaluate to `None`.
    pub fn from_coordinates(x: Option<f32>, y: Option<f32>) -> Option<Direction> {
        // inputs come from the amethyst input manager
        let maybe_x = Direction::horizontal(x.unwrap_or(0.0));
        let maybe_y = Direction::vertical(y.unwrap_or(0.0));

        // if there's input on the horizontal axis, try to combine it with any vertical
        // input, otherwise use any vertical input
        maybe_x.map(|x_dir| x_dir.combine(&maybe_y)).or(maybe_y)
    }

    /// This method lets us use:
    ///     transform.set_rotation_2d(direction.direction_to_radians())
    /// It will rotate the transform on the z-axis to face the given direction.
    /// The transform rotation API uses radians. these values were calculated in
    /// python with `rad = lambda x: (x * math.pi) / 180` and then passing in
    /// degrees (e.g. `rad(90)`).
    pub fn direction_to_radians(self) -> f32 {
        match self {
            Up => 0.0,
            RightUp => -FRAC_PI_4,
            LeftUp => FRAC_PI_4,
            Left => FRAC_PI_2,
            Down => PI,
            LeftDown => 2.356_194_5,
            Right => -FRAC_PI_2,
            RightDown => -2.356_194_5,
            Mouse(manual_dir) => manual_dir.radians,
        }
    }
}
