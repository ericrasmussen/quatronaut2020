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

// this system is concerned only with lasers that have already been spawned.
// the entity exists but the transform needs to be continuously updated based
// on the direction.
// if it collides with a border it should also be destroyed.
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
            // constant laser speed.. still shouldn't be hardcoded though.
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
