#[derive(Debug)]
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

use self::Direction::*;

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
}
