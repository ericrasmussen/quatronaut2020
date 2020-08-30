use amethyst::ecs::{storage::DenseVecStorage, Component};

use serde::{Deserialize, Serialize};

/// This represents everything we need to know about one level in order
/// to build it, track victory conditions, track any special required
/// clean up, and determine what happens if the player finishes the level
#[derive(Clone, Debug)]
pub struct LevelMetadata {
    // number of enemies in the level
    enemy_count: i32,

    layout: Vec<EntityRecord>,

    // TODO: post-refactor this might not make sense
    cleanup_complete: bool,
}

impl LevelMetadata {
    pub fn new(enemy_count: i32, layout: Vec<EntityRecord>) -> LevelMetadata {
        LevelMetadata {
            enemy_count,
            layout,
            cleanup_complete: false,
        }
    }

    // this seems unnecessary, but world resources are easier to modify than
    // to replace completely. this is the simplest solution for now
    pub fn replace_self_with(&mut self, new_metadata: LevelMetadata) {
        self.enemy_count = new_metadata.enemy_count;
        self.layout = new_metadata.layout;
        self.cleanup_complete = new_metadata.cleanup_complete;
    }

    pub fn enemy_destroyed(&mut self) {
        self.enemy_count -= 1;
    }

    pub fn all_enemies_defeated(&self) -> bool {
        self.enemy_count <= 0
    }

    pub fn ready_for_level_transition(&self) -> bool {
        self.all_enemies_defeated() && self.cleanup_complete
    }

    pub fn mark_cleanup_complete(&mut self) {
        self.cleanup_complete = true;
    }

    pub fn get_layout(&self) -> &[EntityRecord] {
        self.layout.as_slice()
    }
}

impl Default for LevelMetadata {
    fn default() -> Self {
        LevelMetadata {
            enemy_count: 0,
            layout: Vec::new(),
            cleanup_complete: true,
        }
    }
}

impl Component for LevelMetadata {
    type Storage = DenseVecStorage<Self>;
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

    let mut enemy_count = 0;

    for (y_index, r) in rows.iter().enumerate() {
        for (x_index, s) in r.chars().enumerate() {
            let entity = match s {
                'F' => {
                    enemy_count += 1;
                    Some(EntityType::FlyingEnemy)
                },
                'S' => {
                    enemy_count += 1;
                    Some(EntityType::SquareEnemy)
                },
                'B' => {
                    enemy_count += 1;
                    Some(EntityType::Boss)
                },

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

    LevelMetadata::new(enemy_count, records)
}

fn get_coordinates(x_grid_pos: usize, y_grid_pos: usize) -> (f32, f32) {
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
