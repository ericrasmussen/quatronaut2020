use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use rand::distributions::{Distribution, Standard};

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
}

use Direction::*;

/// Allows callers to randomly generate directions for spawning
impl Distribution<Direction> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        // randomly chooses a number from 1-8 (inclusive low/exclusvie high)
        let n: u32 = rng.gen_range(1, 9);
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
    // the system part of the ECS will receive pos/neg horizontal and pos/neg vertical
    // input from the player. on a keyboard the values are typically:
    // negative: -1.0
    // neutral:   0.0
    // positive:  1.0
    // an analog stick may provide varying degrees of pos/neg, but it should not affect
    // this game.
    pub fn horizontal(x: f32) -> Option<Direction> {
        if x < 0.0 {
            Some(Left)
        } else if x > 0.0 {
            Some(Right)
        } else {
            None
        }
    }

    pub fn vertical(y: f32) -> Option<Direction> {
        if y < 0.0 {
            Some(Down)
        } else if y > 0.0 {
            Some(Up)
        } else {
            None
        }
    }

    // this approach is a little messy. we're assuming it's only ever used
    // to combine horizontal with vertical. if it's not used that way then it's
    // meaningless, because we can't have RightRight, or RightLeft, and so on.
    pub fn combine(self, other: &Option<Direction>) -> Direction {
        match (self, other) {
            (Right, Some(Up)) => RightUp,
            (Right, Some(Down)) => RightDown,
            (Left, Some(Up)) => LeftUp,
            (Left, Some(Down)) => LeftDown,
            (x, _) => x,
        }
    }

    pub fn from_coordinates(x: Option<f32>, y: Option<f32>) -> Option<Direction> {
        // inputs come from the amethyst input manager
        let maybe_x = Direction::horizontal(x.unwrap_or(0.0));
        let maybe_y = Direction::vertical(y.unwrap_or(0.0));

        // if there's input on the horizontal axis, try to combine it with any vertical
        // input, otherwise use any vertical input
        maybe_x.map(|x_dir| x_dir.combine(&maybe_y)).or(maybe_y)
    }

    // the rotation API uses radians. these values were calculated in python
    // with `rad = lambda x: (x * math.pi) / 180` and then passing in degrees
    // (e.g. `rad(90)`). note that it should be used with `set_rotation` on
    // transforms because it is an absolute value pointing in one direction
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
        }
    }
}
