/// This struct is used by systems that update transforms by making
/// it easy to restrict movement to a specified playable area. This
/// is intended for use by `gameplay.rs` and should be based on the
/// screen dimensions.
use amethyst::ecs::{storage::DenseVecStorage, Component};

pub enum ClampDimension {
    ClampX,
    ClampY,
}

pub struct PlayableArea {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
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
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> PlayableArea {
        PlayableArea {
            min_x,
            max_x,
            min_y,
            max_y,
        }
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
