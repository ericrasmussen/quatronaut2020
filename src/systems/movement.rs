use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use crate::{
    components::movement::Movement,
    entities::player::Player,
    resources::{audio::Sounds, playablearea::PlayableArea},
};

use std::f32::consts::PI;

use log::debug;

#[derive(SystemDesc)]
pub struct MovementTrackingSystem;

impl<'s> System<'s> for MovementTrackingSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Movement>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (transforms, mut movements, players): Self::SystemData) {
        for (movement, transform) in (&mut movements, &transforms).join() {
            for (_player, player_transform) in (&players, &transforms).join() {
                // this updates the x and y velocities on the enemy struct, which
                // can be used in another system to modify the transform
                // we can't modify it here because we can't take ownership of mut
                // transforms in the outer join and still get player transforms in the
                // inner join
                movement.next_move(
                    player_transform.translation().x,
                    player_transform.translation().y,
                    player_transform.translation().z,
                    transform.translation().x,
                    transform.translation().y,
                );
            }
        }
    }
}

// now we can update the transform
#[derive(SystemDesc)]
pub struct TransformUpdateSystem;

#[allow(clippy::type_complexity)]
impl<'s> System<'s> for TransformUpdateSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Movement>,
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        Read<'s, PlayableArea>,
    );

    fn run(
        &mut self,
        (mut transforms, mut movements, time, entities, storage, sounds, audio_output, playable_area): Self::SystemData,
    ) {
        for (movement, enemy_entity, enemy_transform) in (&mut movements, &entities, &mut transforms).join() {
            enemy_transform.prepend_translation_x(movement.velocity_x * time.delta_seconds());
            enemy_transform.prepend_translation_y(movement.velocity_y * time.delta_seconds());

            // these values should be based on game dimensions. the check is needed
            // for enemies that move off screen before getting hit
            let x = enemy_transform.translation().x;
            let y = enemy_transform.translation().y;

            // maybe TODO: smooth rotation? or for projectiles at least,
            // rotated before being spawned
            if let Some(player_vec) = movement.locked_direction {
                if !movement.already_rotated {
                    let dir = player_vec - enemy_transform.translation();
                    let angle = dir.y.atan2(dir.x);
                    let angle_facing = angle - (90.0 * PI / 180.0);
                    enemy_transform.set_rotation_2d(angle_facing);
                    if let Some(sound_type) = movement.launch_sound {
                        sounds.play_sound(sound_type, &storage, audio_output.as_deref());
                    }
                    movement.already_rotated = true;
                }
            }

            // the .delete here is the step that actually removes the enemy
            if playable_area.out_of_bounds(x, y) && entities.delete(enemy_entity).is_ok() {
               debug!("enemy out of bounds");
            }
        }
    }
}
