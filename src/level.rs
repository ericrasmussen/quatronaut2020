use amethyst::ecs::{storage::DenseVecStorage, Component};

use serde::{Deserialize, Serialize};

// we need systems and the update method to work together to handle
// all aspects of clearing a level. a resource of this type can be used
// to track all of the conditions
#[derive(Clone, Debug)]
pub struct LevelComplete {
    // whether or not the player succeeded at finishing the level
    pub success: bool,

    pub cleanup_complete: bool,
}

impl LevelComplete {
    pub fn ready_for_next_level(&self) -> bool {
        self.success && self.cleanup_complete
    }

    pub fn start_over(&mut self) {
        self.success = false;
        self.cleanup_complete = false;
    }
}

// the default condition is for the level to be complete,
// indicating the game is ready for the next level
impl Default for LevelComplete {
    fn default() -> Self {
        LevelComplete {
            success: true,
            cleanup_complete: true,
        }
    }
}

impl Component for LevelComplete {
    type Storage = DenseVecStorage<Self>;
}

// TODO: use slices here for the intermediary type
// e.g. LevelConfig<'a>(&'a [&'a [&'a str]])
#[derive(Debug, Deserialize, Serialize)]
pub struct LevelConfig {
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum EntityType {
    FlyingEnemy,
    BlobEnemy,
    Player,
}

// entity to create, x coordinate, y coordinate
pub type EntityRecord = (EntityType, f32, f32);

// type alias for levels
pub type Levels = Vec<Vec<EntityRecord>>;

// loop through our grid to get a vector containing only entities
// and transform coordinates
pub fn get_level_entities(rows: &mut Vec<String>) -> Vec<EntityRecord> {
    let mut records = Vec::new();

    // make sure we reverse because y=0 is the bottom of the screen
    rows.reverse();

    for (y_index, r) in rows.iter().enumerate() {
        for (x_index, s) in r.chars().enumerate() {
            let entity = match s {
                'F' => Some(EntityType::FlyingEnemy),
                'B' => Some(EntityType::BlobEnemy),
                'P' => Some(EntityType::Player),
                _ => None,
            };

            // coordinates for transform component
            let (x, y) = get_coordinates(x_index, y_index);

            if let Some(e) = entity {
                records.push((e, x, y));
            }
        }
    }

    // return the vector of records
    records
}

fn get_coordinates(x_grid_pos: usize, y_grid_pos: usize) -> (f32, f32) {
    // this obviously shouldn't be hardcoded and will need to be changed when there
    // are assets to work with.
    // this essentially computes a percentage of width and height based on the length
    // of each string (horizontal position) and index of each row (vertical position)
    // and then multiplies it by width and height of our screen dimensions to pick
    // coordinates usable for transform components
    // these come from ScreenDimensions and should use that resource if possible
    let width = 2880.0;
    let height = 1710.0;
    let str_len = 50.0;
    let num_rows = 25.0;

    let x = (x_grid_pos as f32 / str_len) * width;
    let y = (y_grid_pos as f32 / num_rows) * height;

    (x, y)
}

// this is beginning to feel like a bundle... maybe we include a level bundle to get
// all the config files. if it's not a bundle then we can
// derive serialize/deserialize for serde and load it that way from a config
pub fn get_all_levels(mut level_config: LevelConfig) -> Levels {
    level_config.rows.reverse();

    let mut levels_vec = Vec::new();

    for mut level in level_config.rows.iter_mut() {
        let next_level = get_level_entities(&mut level);
        levels_vec.push(next_level);
    }
    levels_vec.reverse();

    levels_vec
}
