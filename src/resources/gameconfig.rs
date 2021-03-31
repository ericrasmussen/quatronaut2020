/// Struct to collect resources that are passed around all over.
/// menu.rs needs to set and pass these to a new gameplay.rs game
/// transition.rs needs to potentially change and pass them back to gameplay.rs
/// gameplay.rs needs them to run the game
use crate::resources::audio::SoundConfig;
use crate::resources::level::{LevelConfig, Levels};

/// This tracks whether we're in a level, transitioning between levels,
/// or if we've finished all of them.
#[derive(Clone, Debug, PartialEq)]
pub enum GameplayMode {
    LevelMode,
    TransitionMode,
    CompletedMode,
}

#[derive(Clone, Debug)]
pub struct GameConfig {
    pub level_config: LevelConfig,
    pub sound_config: SoundConfig,
    pub current_levels: Levels,
    pub gameplay_mode: GameplayMode,
    pub immortal_hyper_mode: bool,
}
