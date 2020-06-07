use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
};

use crate::entities::{
    laser::{spawn_laser, Laser},
    player::Player,
};
use amethyst_rendy::sprite::SpriteRender;

//use log::info;

#[derive(SystemDesc)]
pub struct PlayerSystem;

// this system is likely too complicated, but it's not clear if there's a benefit
// to breaking some of it into separate systems (for instance, one system to track
// input, another to modify the transform, another to spawn lasers, etc)
#[allow(clippy::type_complexity)]
impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
        Entities<'s>,
        ReadStorage<'s, SpriteRender>,
        ReadExpect<'s, LazyUpdate>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut characters, input, entities, sprites, lazy_update, time): Self::SystemData) {
        for (character, transform, sprite) in (&mut characters, &mut transforms, &sprites).join() {
            // the input names here are defined in config/bindings.ron.
            // in general 0 is no movement, 1 is positive, and -1 is negative
            // (analog sticks might have other degrees of > 0 and < 0)
            let movement_x = input.axis_value("x_axis");
            let movement_y = input.axis_value("y_axis");

            // update the x and y coordinates based on current input (if there is
            // no movement then new_x and new_y will equal 0 and the transform
            // coordinates will not be changed)
            if let Some(x_amt) = movement_x {
                let new_x = time.delta_seconds() * x_amt * character.get_speed();
                transform.set_translation_x(transform.translation().x + new_x);
            }

            if let Some(y_amt) = movement_y {
                let new_y = time.delta_seconds() * y_amt * character.get_speed();
                transform.set_translation_y(transform.translation().y + new_y);
            }

            // this tracks whether or not the player is shooting. it makes sense to stay
            // here for now, mostly to avoid weird issues in the future that might allow
            // firing lasers without a player entity
            let laser_x = input.axis_value("x_laser");
            let laser_y = input.axis_value("y_laser");

            // this computes Some(laser_with_direction) or None, based on input
            // (e.g. right and up arrows will create Some(Laser::new(RightUp)))
            let maybe_laser = Laser::from_coordinates(laser_x, laser_y, character.laser_speed);

            // cloning the sprite sheet here is pretty hacky...
            // it should be a prefab or shared resource of some kind, not tied
            // to the sprite sheet the player is using
            if let Some(laser) = maybe_laser {
                if character.can_fire(time.delta_seconds()) {
                    spawn_laser(sprite.clone().sprite_sheet, laser, &transform, &entities, &lazy_update);
                }
            }
        }
    }
}
