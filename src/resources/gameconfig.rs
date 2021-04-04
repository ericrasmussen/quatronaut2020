//! Struct to collect resources that are passed around all over.
//! - menu.rs needs to set and pass these to a new gameplay.rs game
//! - transition.rs needs to potentially change and pass them back to gameplay.rs
//! - gameplay.rs needs them to run the game
//!
//! It would also be possible to insert this into the world and look it
//! up as needed, but when debugging I've found it really convenient to have
//! some known amount of config that I can access and check in code (without
//! having to look in storage).
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

/// This keeps track of all our levels, all our sound effects,
/// the current levels (based on the player's progression so far),
/// the current gameplay mode, and whether or not the player is
/// an otherworldly immortal circle come to rain destruction on its foes.
#[derive(Clone, Debug)]
pub struct GameConfig {
    pub level_config: LevelConfig,
    pub sound_config: SoundConfig,
    pub current_levels: Levels,
    pub gameplay_mode: GameplayMode,
    pub immortal_hyper_mode: bool,
}
