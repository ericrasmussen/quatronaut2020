use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, WriteStorage},
};

use crate::{
    components::{collider::Collider, launcher::Projectile},
    entities::{enemy::Enemy, player::Player},
};
use log::info;

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
        for (player_entity, _player, player_transform, player_collider) in
            (&entities, &players, &transforms, &colliders).join()
        {
            let player_aabb = player_collider
                .aabb_from_coordinates(player_transform.translation().x, player_transform.translation().y);

            for (_enemy_entity, _enemy, enemy_transform, enemy_collider) in
                (&entities, &enemies, &transforms, &colliders).join()
            {
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

#[derive(SystemDesc)]
pub struct ProjectileHitSystem;

impl<'s> System<'s> for ProjectileHitSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Projectile>,
        ReadStorage<'s, Collider>,
        Entities<'s>,
    );

    // we don't need `player` here, though if we add health it'd be useful. keeping for now
    // until deciding
    fn run(&mut self, (transforms, players, projectiles, colliders, entities): Self::SystemData) {
        for (player_entity, _player, player_transform, player_collider) in
            (&entities, &players, &transforms, &colliders).join()
        {
            let player_aabb = player_collider
                .aabb_from_coordinates(player_transform.translation().x, player_transform.translation().y);

            for (projectile_entity, _projectile, projectile_transform, projectile_collider) in
                (&entities, &projectiles, &transforms, &colliders).join()
            {
                let collides = projectile_collider.intersects(
                    projectile_transform.translation().x,
                    projectile_transform.translation().y,
                    &player_aabb,
                );

                if collides {
                    // we probably don't actually want to delete the player instantly,
                    // but how else will we artificially inflate difficulty in a short game
                    info!("player was hit!");
                    entities.delete(player_entity).unwrap();

                    // the projectile for sure is no longer needed after contact
                    entities.delete(projectile_entity).unwrap();
                }

                let trans = projectile_transform.translation();
                if trans.x < 0.0 || trans.x > 2500.0 || trans.y < 0.0 || trans.y > 2500.0 {
                    entities.delete(projectile_entity).unwrap();
                }
            }
        }
    }
}
