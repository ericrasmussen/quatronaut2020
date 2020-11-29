use rand::distributions::{Distribution, Standard};
use serde::{Deserialize, Serialize};

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

    pub fn get_layout(&self) -> &[EntityRecord] {
        self.layout.as_slice()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LevelConfig {
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum EntityType {
    FlyingEnemy,
    SquareEnemy,
    Boss,
    Player,
}

/// Allows callers to randomly generate entity types for spawning
impl Distribution<EntityType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> EntityType {
        // randomly chooses 1, 2, or 3
        let n: u32 = rng.gen_range(1, 4);
        match n {
            1 => EntityType::FlyingEnemy,
            2 => EntityType::SquareEnemy,
            _ => EntityType::Boss,
        }
    }
}

// entity to create, x coordinate, y coordinate
pub type EntityRecord = (EntityType, f32, f32);

// type alias for levels
pub type Levels = Vec<LevelMetadata>;

// loop through our grid to get a vector containing only entities
// and transform coordinates and other level
fn get_level_entities(rows: &mut Vec<String>) -> LevelMetadata {
    // make sure we reverse because y=0 is the bottom of the screen,
    // but the level config is ordered top to bottom
    rows.reverse();

    let mut records = Vec::new();

    for (y_index, r) in rows.iter().enumerate() {
        for (x_index, s) in r.chars().enumerate() {
            let entity = match s {
                'F' => Some(EntityType::FlyingEnemy),
                'S' => Some(EntityType::SquareEnemy),
                'B' => Some(EntityType::Boss),

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

    LevelMetadata::new(records)
}

// gets a percentage of width/height where an enemy should be rendered in the play area
fn get_coordinates(x_grid_pos: usize, y_grid_pos: usize) -> (f32, f32) {
    // this essentially computes a percentage of width and height based on the length
    // of each string (horizontal position) and index of each row (vertical position)
    // and then multiplies it by width and height of our screen dimensions to pick
    // coordinates usable for transform components
    // these come from ScreenDimensions and should use that resource if possible
    let str_len = 50.0;
    let num_rows = 25.0;

    let x = x_grid_pos as f32 / str_len;
    let y = y_grid_pos as f32 / num_rows;

    (x, y)
}

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
