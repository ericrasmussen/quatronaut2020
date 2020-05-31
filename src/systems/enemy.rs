use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

use crate::entities::{enemy::Enemy, player::Player};

//use log::info;

#[derive(SystemDesc)]
pub struct EnemyTrackingSystem;

// this system is likely too complicated, but it's not clear if there's a benefit
// to breaking some of it into separate systems (for instance, one system to track
// input, another to modify the transform, another to spawn lasers, etc)
impl<'s> System<'s> for EnemyTrackingSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Enemy>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
    );

    fn run(&mut self, (transforms, mut enemies, players, _time): Self::SystemData) {
        // seems like we should have another way to get to the player transform since
        // this always be a for loop for a single player. and if it's not, enemies would be
        // moving at high speeds towards groups of players, or not at all if players are
        // in opposite directions
        for (enemy, enemy_transform) in (&mut enemies, &transforms).join() {
            for (_player, player_transform) in (&players, &transforms).join() {
                // this updates the x and y velocities on the enemy struct, which
                // can be used in another system to modify the transform
                // we can't modify it here because we can't take ownership of mut
                // transforms in the outer join and still get player transforms in the
                // inner join
                enemy.move_towards(
                    player_transform.translation().x,
                    player_transform.translation().y,
                    enemy_transform.translation().x,
                    enemy_transform.translation().y,
                );
            }
        }
    }
}

// now we can update the transform
#[derive(SystemDesc)]
pub struct EnemyMoveSystem;

// this system is likely too complicated, but it's not clear if there's a benefit
// to breaking some of it into separate systems (for instance, one system to track
// input, another to modify the transform, another to spawn lasers, etc)
impl<'s> System<'s> for EnemyMoveSystem {
    type SystemData = (WriteStorage<'s, Transform>, ReadStorage<'s, Enemy>, Read<'s, Time>);

    fn run(&mut self, (mut transforms, enemies, time): Self::SystemData) {
        // seems like we should have another way to get to the player transform since
        // this always be a for loop for a single player. and if it's not, enemies would be
        // moving at high speeds towards groups of players, or not at all if players are
        // in opposite directions
        for (enemy, enemy_transform) in (&enemies, &mut transforms).join() {
            enemy_transform.prepend_translation_x(enemy.velocity_x * time.delta_seconds());
            enemy_transform.prepend_translation_y(enemy.velocity_y * time.delta_seconds());
        }
    }
}
