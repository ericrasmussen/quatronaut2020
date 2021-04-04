//! This module contains an extremely advanced level editor
//! guaranteed to provide amazement (or is it amusement?). It also
//! loads the levels from a `.ron` file and makes it easier for
//! the `GameConfig` struct to keep track of all level related data.
use serde::{Deserialize, Serialize};

/// All the entity types we allow in our text-based level editor.
/// (assets/config/levels.ron)
#[derive(Debug, Clone)]
pub enum EntityType {
    FlyingEnemy,
    SquareEnemy,
    Boss,
    Player,
}

/// The entity to create and a percentage (x, y) representing where
/// they were in the text level config.
/// e.g. in a row with `"P   F"`, P's x value is 20% from the left, and
/// F is 100% from the left.
pub type EntityRecord = (EntityType, f32, f32);

/// This represents everything we need to know about one level in order
/// to build it, track victory conditions, track any special required
/// clean up, and determine what happens if the player finishes the level
#[derive(Clone, Debug)]
pub struct LevelMetadata {
    layout: Vec<EntityRecord>,
}

impl LevelMetadata {
    pub fn new(layout: Vec<EntityRecord>) -> LevelMetadata {
        LevelMetadata { layout }
    }

    /// Get the level layout
    pub fn get_layout(&self) -> &[EntityRecord] {
        self.layout.as_slice()
    }
}

/// Quatronaut has small levels with a constrained play area,
/// and large levels with a much wider play area. This lets us
/// track them separately so we can make decisions about play
/// area and level transitions. This particular representation is mostly
/// rows of rows of strings so it can be deserialized from a config file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelConfig {
    pub small_levels: Vec<Vec<String>>,
    pub large_levels: Vec<Vec<String>>,
}

/// This is the main code representation of our fully loaded levels,
/// where we can see if we're using small levels, and then get
/// all the entities and their relative positions for a given level.
#[derive(Clone, Debug)]
pub struct Levels {
    pub small_levels: Vec<LevelMetadata>,
    pub large_levels: Vec<LevelMetadata>,
    use_small_levels: bool,
}

/// This status enum is used by `Levels` when returning level metadata
/// along with the status (i.e. are we returning a small level, a
/// large level, a small level transitioning into a large level, or are
/// we all done).
#[derive(Debug)]
pub enum LevelStatus {
    SmallLevel(LevelMetadata),
    LargeLevel(LevelMetadata),
    TransitionTime(LevelMetadata),
    AllDone,
}

impl Levels {
    /// This needs to return at least three variants:
    ///   1) the next small level
    ///   2) an indicator we should transition to the large bg
    ///   3) the next large level
    ///   4) something to indicate levels are exhausted
    pub fn pop(&mut self) -> Option<LevelStatus> {
        // if we have any small levels left, use that vec
        if self.use_small_levels {
            match self.small_levels.pop() {
                Some(metadata) => {
                    // we're on the last small level and want to transition
                    // we also don't want to use small levels next time
                    if self.small_levels.is_empty() {
                        self.use_small_levels = false;
                        Some(LevelStatus::TransitionTime(metadata))
                    } else {
                        Some(LevelStatus::SmallLevel(metadata))
                    }
                },
                // this can only be reached if there are no small levels at all
                None => None,
            }
        } else {
            match self.large_levels.pop() {
                Some(metadata) => Some(LevelStatus::LargeLevel(metadata)),
                None => Some(LevelStatus::AllDone),
            }
        }
    }
}

/// Loop through our grid to get a vector containing only entities
/// and their relative positions in the level.
fn get_level_entities(rows: &mut Vec<String>) -> LevelMetadata {
    // make sure we reverse because y=0 is the bottom of the screen,
    // but the level config is ordered top to bottom
    rows.reverse();
    let num_rows = rows.len();

    let mut records = Vec::new();

    for (y_index, r) in rows.iter().enumerate() {
        let num_columns = r.len();
        for (x_index, s) in r.chars().enumerate() {
            let entity = match s {
                'F' => Some(EntityType::FlyingEnemy),
                'S' => Some(EntityType::SquareEnemy),
                'B' => Some(EntityType::Boss),
                'P' => Some(EntityType::Player),
                _ => None,
            };

            // coordinates for transform component
            let (x, y) = get_coordinates(x_index, y_index, num_rows, num_columns);

            if let Some(e) = entity {
                records.push((e, x, y));
            }
        }
    }

    LevelMetadata::new(records)
}

/// Helper that gets a percentage of width/height that helps us map the position in
/// the text level config to a real (x, y) coordinate in the game area.
fn get_coordinates(x_grid_pos: usize, y_grid_pos: usize, num_rows: usize, num_columns: usize) -> (f32, f32) {
    // this essentially computes a percentage of width and height based on the length
    // of each string (horizontal position) and index of each row (vertical position)
    // and then multiplies it by width and height of our screen dimensions to pick
    // coordinates usable for transform components
    // these come from ScreenDimensions and should use that resource if possible
    let x = x_grid_pos as f32 / num_columns as f32;
    let y = y_grid_pos as f32 / num_rows as f32;

    (x, y)
}

/// Top-level method to read in the level config and give us all the `Levels`.
pub fn get_all_levels(level_config: LevelConfig) -> Levels {
    Levels {
        small_levels: extract_levels(level_config.small_levels),
        large_levels: extract_levels(level_config.large_levels),
        use_small_levels: true,
    }
}

/// This method loops over the rows of rows in the config file and makes a
/// new `Vec` with all the metadata. It needs to be reversed because in
/// the level editor we look at row 0 in the list as the top of the level, but
/// the y coordinate 0 position actually starts at the bottom.
fn extract_levels(mut level_rows: Vec<Vec<String>>) -> Vec<LevelMetadata> {
    level_rows.reverse();

    let mut levels_vec = Vec::new();

    for mut level in level_rows.iter_mut() {
        let next_level = get_level_entities(&mut level);
        levels_vec.push(next_level);
    }
    levels_vec.reverse();

    levels_vec
}
