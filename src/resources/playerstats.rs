/// This struct is used for any player metadata that we might need
/// to track, e.g. their current score.
use amethyst::ecs::{storage::DenseVecStorage, Component};

#[derive(Debug)]
pub struct PlayerStats {
    score: i32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats { score: 0 }
    }
}

impl PlayerStats {
    pub fn add_to_score(&mut self, value: i32) {
        self.score += value;
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }
}

impl Component for PlayerStats {
    type Storage = DenseVecStorage<Self>;
}
