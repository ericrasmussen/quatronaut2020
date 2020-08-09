use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
};

use crate::{
    entities::{enemy::Enemy, player::Player},
    level::EnemyCount,
};

use std::f32::consts::PI;

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

        // TODO: use the player position resource here
        for (enemy, enemy_transform) in (&mut enemies, &transforms).join() {
            for (_player, player_transform) in (&players, &transforms).join() {
                // this updates the x and y velocities on the enemy struct, which
                // can be used in another system to modify the transform
                // we can't modify it here because we can't take ownership of mut
                // transforms in the outer join and still get player transforms in the
                // inner join
                enemy.next_move(
                    player_transform.translation().x,
                    player_transform.translation().y,
                    player_transform.translation().z,
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
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Enemy>,
        Read<'s, Time>,
        Write<'s, EnemyCount>,
        Entities<'s>,
    );

    // TODO: delete enemies that go way out of bounds. maybe using arena bounds + generous
    // padding. this is necesary because some enemies will continue in one direction forever
    fn run(&mut self, (mut transforms, mut enemies, time, mut enemy_count, entities): Self::SystemData) {
        for (enemy, enemy_entity, enemy_transform) in (&mut enemies, &entities, &mut transforms).join() {
            enemy_transform.prepend_translation_x(enemy.velocity_x * time.delta_seconds());
            enemy_transform.prepend_translation_y(enemy.velocity_y * time.delta_seconds());

            // these values should be based on game dimensions. the check is needed
            // for enemies that move off screen before getting hit
            let x = enemy_transform.translation().x;
            let y = enemy_transform.translation().y;

            // BIG TODO: the intended rotation here isn't working, probably due to some
            // fundamental misunderstanding from the code commenting narrator. for now there's
            // a quick hack to make sure the flying enemies at least point downward, and this can
            // be revisited later
            if let Some(_player_vec) = enemy.locked_direction {
                if !enemy.already_rotated {
                    enemy_transform.prepend_rotation_z_axis(PI);
                    enemy.already_rotated = true;
                    //let enemy_vec = enemy_transform.translation();
                    //let radians = player_vec.dot(&enemy_vec).acos();
                    //info!("prepending rotation for {} radians", radians);
                    //enemy_transform.prepend_rotation_z_axis(radians); // + FRAC_PI_2);
                    //enemy.already_rotated = true;
                }
            }

            let out_of_bounds = x < -500.0 || x > 2500.0 || y < -500.0 || y > 2500.0;

            if out_of_bounds && entities.delete(enemy_entity).is_ok() {
                enemy_count.decrement_by(1);
                //info!("enemy out of bounds");
                //info!("new enemy count is: {}", enemy_count.count);
            }
        }
    }
}
