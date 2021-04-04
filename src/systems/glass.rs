//! This is used for the glass breaking effect when the
//! arcade background "breaks", before the camera zooms out to
//! reveal the widescreen broken background.
use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, System, SystemData, WriteStorage},
};

use crate::{
    components::glass::Glass,
    resources::{direction::Direction, playablearea::PlayableArea},
};

use log::info;

/// This system sends glass flying off in whatever ``glass.direction`` they
/// have, at their given ``glass.speed``. A lot of the code is duplicated
/// from `laser.rs`. Ideally they'd be consolidated into something more generic.
#[derive(SystemDesc)]
pub struct GlassSystem;

impl<'s> System<'s> for GlassSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Glass>,
        Entities<'s>,
        Read<'s, Time>,
        Read<'s, PlayableArea>,
    );

    fn run(&mut self, (mut transforms, glass_shards, entities, time, playable_area): Self::SystemData) {
        for (entity, glass, transform) in (&entities, &glass_shards, &mut transforms).join() {
            // mostly stolen from laser.rs. ideally each glass struct would have a closure
            // for trans.<var> <op> speed, which would then be multiplied by delta seconds here
            let &trans = transform.translation();
            let neg_x = trans.x - glass.speed * time.delta_seconds();
            let neg_y = trans.y - glass.speed * time.delta_seconds();
            let pos_x = trans.x + glass.speed * time.delta_seconds();
            let pos_y = trans.y + glass.speed * time.delta_seconds();

            match &glass.direction {
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
