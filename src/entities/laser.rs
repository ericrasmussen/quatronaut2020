/// This module includes the laser component creation, laser entity,
/// and the Direction enum used for rotating the sprite and determining
/// velocity.
/// If we need to reuse Direction it can be turned into a separate component.
use amethyst::{
    core::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect},
    renderer::{sprite::SpriteSheetHandle, SpriteRender},
};

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use crate::components::cleanup::CleanupTag;
use crate::resources::direction::Direction;

//use log::info;

/// This is the laser component type, used by `spawn_laser` to create new
/// laser entities.
/// The systems/player.rs file determines, based on player input, when to
/// fire lasers.
/// The systems/laser.rs module is responsible for updating the laser position
/// and eventually destroying it. It will eventually be used for collision detection.
#[derive(Debug)]
pub struct Laser {
    pub direction: Direction,
    pub speed: f32,
}

impl Laser {
    pub fn new(direction: Direction, speed: f32) -> Laser {
        Laser { direction, speed }
    }

    // we're receiving two types of inputs that may or may not be directional.
    // we need to decide if they are directional (e.g. Up, or Right and Up),
    // combine the horizontal and vertical directions if possible, and finally
    // create a new laser.
    pub fn from_coordinates(x: Option<f32>, y: Option<f32>, speed: f32) -> Option<Laser> {
        // inputs come from the amethyst input manager
        let maybe_x = Direction::horizontal(x.unwrap_or(0.0));
        let maybe_y = Direction::vertical(y.unwrap_or(0.0));

        // if there's input on the horizontal axis, try to combine it with any vertical
        // input, otherwise use any vertical input
        let maybe_composite = maybe_x.map(|x_dir| x_dir.combine(&maybe_y)).or(maybe_y);

        // once we have determined the one true direction or no
        // direction at all, we can return our Option<Laser>
        match maybe_composite {
            Some(dir) => Some(Laser::new(dir, speed)),
            _ => None,
        }
    }
}

impl Component for Laser {
    type Storage = DenseVecStorage<Self>;
}

// this is used by systems/player.rs to create lasers whenever the player fires
// them. the lazy_update usage is from the space-menace game example and may
// not be required.
// BIG TODO: we should not be borrowing the sprite sheet handle from the player.
// it should be available as a separate game resource and used as a prefab or
// otherwise found in the world. this implementation ties the laser image to
// the sprite sheet being used by the player.
pub fn spawn_laser(
    sprite_sheet_handle: SpriteSheetHandle,
    laser: Laser,
    player_transform: &Transform,
    entities: &Entities,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    // an incorrect sprite number here will lead to a memory leak
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 2,
    };

    let mut transform = player_transform.clone();

    // the rotation API uses radians. these values were calculated in python
    // with `rad = lambda x: (x * math.pi) / 180` and then passing in degrees
    // (e.g. `rad(90)`)
    match laser.direction {
        Direction::Up => transform.set_rotation_2d(0.0),
        Direction::RightUp => transform.set_rotation_2d(-FRAC_PI_4),
        Direction::LeftUp => transform.set_rotation_2d(FRAC_PI_4),
        Direction::Left => transform.set_rotation_2d(FRAC_PI_2),
        Direction::Down => transform.set_rotation_2d(PI),
        Direction::LeftDown => transform.set_rotation_2d(2.356_194_5),
        Direction::Right => transform.set_rotation_2d(-FRAC_PI_2),
        Direction::RightDown => transform.set_rotation_2d(-2.356_194_5),
    };

    let laser_entity: Entity = entities.create();
    let cleanup_tag = CleanupTag {};
    lazy_update.insert(laser_entity, laser);
    lazy_update.insert(laser_entity, cleanup_tag);
    lazy_update.insert(laser_entity, transform);
    lazy_update.insert(laser_entity, sprite_render);
}
