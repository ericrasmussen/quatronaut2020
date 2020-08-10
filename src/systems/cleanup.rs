use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, System, SystemData, Write, WriteStorage},
};

use crate::{entities::player::Player, resources::level::LevelMetadata};

use log::info;

#[derive(SystemDesc)]
pub struct CleanupSystem;

// this system exists to remove the player entity when victory is achieved
// if it happens too suddenly we might do something like a victory condition
// resource that requires both enemy count <= 0 and some time elapsed since
// the last enemy was removed
impl<'s> System<'s> for CleanupSystem {
    type SystemData = (WriteStorage<'s, Player>, Write<'s, LevelMetadata>, Entities<'s>);

    // open question: should lasers be deleted here too? otherwise they may still be flying
    // across the screen when a new level starts
    fn run(&mut self, (players, mut level_metadata, entities): Self::SystemData) {
        if level_metadata.all_enemies_defeated() {
            for (player_entity, _player) in (&entities, &players).join() {
                entities.delete(player_entity).unwrap();
                info!("deleted a player!!!!!");
            }
        }
        if players.is_empty() {
            level_metadata.mark_cleanup_complete();
            //info!("players is empty! wait what");
        }
    }
}
