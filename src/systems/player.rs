//! This module handles all player-based input and uses it to fire lasers
//! and update the player position.
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
    input::{InputHandler, StringBindings},
    window::ScreenDimensions,
};

use crate::entities::{
    laser::{spawn_laser, Laser},
    player::Player,
};

use crate::resources::{
    audio::{SoundType, Sounds},
    direction::{Direction, ManualDirection},
    playablearea::PlayableArea,
};

use amethyst_rendy::sprite::SpriteRender;

use amethyst::winit::MouseButton;

// use log::info;

#[derive(SystemDesc)]
pub struct PlayerSystem;

/// This system is doing too many things, but it's still a relatively small amount
/// of code. It gets information on the movement and laser inputs, then moves the
/// player and spawns lasers (when possible, as determined by the player's configured
/// firing rate, since we wouldn't want lasers spawning every frame)
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
        Read<'s, PlayableArea>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        Option<Read<'s, ScreenDimensions>>,
    );

    fn run(
        &mut self,
        (
            mut transforms,
            mut characters,
            input,
            entities,
            sprites,
            lazy_update,
            time,
            playable_area,
            storage,
            sounds,
            audio_output,
            dimensions,
        ): Self::SystemData,
    ) {
        let dimensions_height = dimensions.expect("panic on missing screen dimensions").height();
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
                let new_x = time.delta_seconds() * x_amt * character.get_speed() + transform.translation().x;
                transform.set_translation_x(playable_area.clamp_x(new_x));
            }

            if let Some(y_amt) = movement_y {
                let new_y = time.delta_seconds() * y_amt * character.get_speed() + transform.translation().y;
                transform.set_translation_y(playable_area.clamp_y(new_y));
            }

            // this tracks whether or not the player is shooting. it makes sense to stay
            // here for now, mostly to avoid weird issues in the future that might allow
            // firing lasers without a player entity
            let laser_x = input.axis_value("x_laser");
            let laser_y = input.axis_value("y_laser");

            // optionally creates a new direction for the player (and possibly laser) based on the mouse
            // click coordinates or the keyboard arrows
            let maybe_direction = if input.mouse_button_is_down(MouseButton::Left) {
                if let Some((x, y)) = input.mouse_position() {
                    // info!("player at ({}, {}) clicked at ({}, {})", transform.translation().x,
                    // transform.translation().y, x, y);
                    let manual = ManualDirection::new(
                        transform.translation().x,
                        transform.translation().y,
                        transform.translation().z,
                        x,
                        dimensions_height - y,
                    );
                    Some(Direction::Mouse(manual))
                } else {
                    None
                }
            } else {
                Direction::from_coordinates(laser_x, laser_y)
            };

            if let Some(dir) = maybe_direction {
                character.direction = dir;
            }
            transform.set_rotation_2d(character.direction.direction_to_radians());

            // this computes Some(laser_with_direction) or None, based on input
            // (e.g. right and up arrows will create Some(Laser::new(RightUp)))
            let maybe_laser = if let Some(d) = maybe_direction {
                Some(Laser::from_dir(d, character.laser_speed))
            } else {
                Laser::from_coordinates(laser_x, laser_y, character.laser_speed)
            };

            // cloning the sprite sheet here is pretty hacky...
            // it should be a prefab or shared resource of some kind, not tied
            // to the sprite sheet the player is using
            if let Some(laser) = maybe_laser {
                if character.can_fire(time.delta_seconds()) {
                    spawn_laser(sprite.clone().sprite_sheet, laser, &transform, &entities, &lazy_update);
                    // if we created a laser, play a laser sound
                    sounds.play_sound(SoundType::PlayerBlaster, &storage, audio_output.as_deref());
                }
            }
        }
    }
}
