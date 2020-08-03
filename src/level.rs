use serde::{Deserialize, Serialize};

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
    let width = 1024.0 * 2.0;
    let height = 768.0 * 2.0;
    let str_len = 11.0;
    let num_rows = 7.0;

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
