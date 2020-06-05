use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, WriteStorage},
};

use crate::entities::{enemy::Enemy, player::Player};
use crate::components::collider::Collider;
//use log::info;

// big TODO: as this system gets more complicated, at some point it'll probably
// be worth using ncollide's broad phase collision, which would let us consolidate
// this and collision.rs.
#[derive(SystemDesc)]
pub struct AttackedSystem;

impl<'s> System<'s> for AttackedSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Enemy>,
        ReadStorage<'s, Collider>,
        Entities<'s>,
    );

    // we don't need `player` here, though if we add health it'd be useful. keeping for now
    // until deciding
    fn run(&mut self, (transforms, players, enemies, colliders, entities): Self::SystemData) {
        for (player_entity, _player, player_transform, player_collider) in (&entities, &players, &transforms, &colliders).join() {

            let player_aabb = player_collider.aabb_from_coordinates(
                player_transform.translation().x,
                player_transform.translation().y
            );

            for (_enemy_entity, _enemy, enemy_transform, enemy_collider) in (&entities, &enemies, &transforms, &colliders).join() {

                let collides = enemy_collider.intersects(
                    enemy_transform.translation().x,
                    enemy_transform.translation().y,
                    &player_aabb,
                );

                if collides {
                // this should be a call to some enemy method for reducing health
                    entities.delete(player_entity).unwrap();
                }
            }
        }
    }
}
