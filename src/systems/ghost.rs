/// This controls the ghost images that apppear after an enemy is defeated
use amethyst::{
    core::{timing::Time, Transform},
    core::math::Vector3,
    derive::SystemDesc,
    ecs::{Entities, Join, Read, System, SystemData, WriteStorage},
};

use crate::entities::enemy::Ghost;


#[derive(SystemDesc)]
pub struct GhostSystem;

impl<'s> System<'s> for GhostSystem {
    type SystemData = (
        WriteStorage<'s, Ghost>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut ghosts, mut transforms, entities, time): Self::SystemData) {
        for (ghost_entity, ghost, transform) in (&entities, &mut ghosts, &mut transforms).join() {
            // no need to keep the ghost around past this point
            // (we don't want a ghost of a ghost)
            if ghost.is_done_fading() {
                entities.delete(ghost_entity).unwrap()
            } else {
                let next_scale = ghost.next_scale(transform.scale().x, time.delta_seconds());
                transform.set_scale(Vector3::from_element(next_scale));
            }
        }
    }
}
