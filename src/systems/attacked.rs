/// These systems deal with conditions that delete players, such as being hit by
/// an enemy or a projectile. Right now there are only two cases, but if there are
/// ever three or more then this should probably send a "player hit" event so it
/// can be handled in one place.
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use crate::{
    components::{collider::Collider, launcher::Projectile},
    entities::{enemy::Enemy, player::Player},
    resources::audio::{SoundType, Sounds},
};
use log::info;

#[derive(SystemDesc)]
pub struct AttackedSystem;

impl<'s> System<'s> for AttackedSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Enemy>,
        ReadStorage<'s, Collider>,
        Entities<'s>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    // note that `player` is needed here as part of the query to ensure we're
    // dealing with player entities (otherwise we'd be checking every game entity with projectiles and
    // colliders)
    fn run(
        &mut self,
        (transforms, players, enemies, colliders, entities, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (player_entity, player, player_transform, player_collider) in
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

                if collides && !player.invulnerable {
                    sounds.play_sound(SoundType::PlayerDeath, &storage, audio_output.as_deref());
                    entities.delete(player_entity).unwrap();
                    info!("player was hit!");
                }
            }
        }
    }
}

#[derive(SystemDesc)]
pub struct ProjectileHitSystem;

impl<'s> System<'s> for ProjectileHitSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Projectile>,
        ReadStorage<'s, Collider>,
        Entities<'s>,
    );

    // note that `player` is needed here as part of the query to ensure we're
    // dealing with player entities (otherwise we'd be checking every game entity with projectiles and
    // colliders)
    fn run(&mut self, (transforms, players, projectiles, colliders, entities): Self::SystemData) {
        for (player_entity, player, player_transform, player_collider) in
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
                    // TODO: this is also a game over event condition
                    if !player.invulnerable {
                        info!("player was hit!");
                        entities.delete(player_entity).unwrap();
                    }
                    // the projectile for sure is no longer needed after contact
                    entities.delete(projectile_entity).unwrap();
                }

                let trans = projectile_transform.translation();
                // 2880.0 x 1710.0
                if trans.x < -5.0 || trans.x > 2900.0 || trans.y < -5.0 || trans.y > 2000.0 {
                    entities.delete(projectile_entity).unwrap();
                }
            }
        }
    }
}
