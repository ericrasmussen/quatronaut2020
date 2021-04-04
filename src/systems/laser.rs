//! The player character has exactly one weapon at their disposal:
//! a trusty laser blaster mounted on their face. Once a laser is
//! fired, it continues moving in its given `Direction` until it
//! collides with an enemy (see `collision.rs`) or it goes out of
//! bounds.
use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, System, SystemData, WriteStorage},
};

use crate::{
    entities::laser::Laser,
    resources::{direction::Direction, playablearea::PlayableArea},
};

use log::info;

/// The main responsibility of `LaserSystem` is to update the laser's
/// transform component based on its speed, direction, and delta time.
/// `collision.rs` may destroy these lasers if they hit enemies, otherwise
/// this system will delete them whenever they travel outside the playing area.
#[derive(SystemDesc)]
pub struct LaserSystem;

impl<'s> System<'s> for LaserSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Laser>,
        Entities<'s>,
        Read<'s, Time>,
        Read<'s, PlayableArea>,
    );

    fn run(&mut self, (mut transforms, lasers, entities, time, playable_area): Self::SystemData) {
        for (entity, laser, transform) in (&entities, &lasers, &mut transforms).join() {
            let &trans = transform.translation();
            let neg_x = trans.x - laser.speed * time.delta_seconds();
            let neg_y = trans.y - laser.speed * time.delta_seconds();
            let pos_x = trans.x + laser.speed * time.delta_seconds();
            let pos_y = trans.y + laser.speed * time.delta_seconds();

            // probably no reason to compute this every frame for every laser
            // it'd be easier to have the laser track `.next_change` or something
            // similar
            match &laser.direction {
                Direction::Left => {
                    transform.set_translation_x(neg_x);
                },
                Direction::Right => {
                    transform.set_translation_x(pos_x);
                },
                Direction::Up => {
                    transform.set_translation_y(pos_y);
                },
                Direction::Down => {
                    transform.set_translation_y(neg_y);
                },
                Direction::RightUp => {
                    transform.set_translation_x(pos_x);
                    transform.set_translation_y(pos_y);
                },
                Direction::LeftUp => {
                    transform.set_translation_x(neg_x);
                    transform.set_translation_y(pos_y);
                },
                Direction::LeftDown => {
                    transform.set_translation_x(neg_x);
                    transform.set_translation_y(neg_y);
                },
                Direction::RightDown => {
                    transform.set_translation_x(pos_x);
                    transform.set_translation_y(neg_y);
                },
            }

            if playable_area.out_of_bounds(trans.x, trans.y) {
                let deleted = entities.delete(entity);

                if let Err(msg) = deleted {
                    info!("A terrible error has occured: {:?}", msg)
                }
            }
        }
    }
}
