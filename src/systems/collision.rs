//! This module detects laser collisions with enemies so they can take
//! damage. See `attacked.rs` for collisions with the player.
use nalgebra::{Isometry2, Vector2};
use ncollide2d::{bounding_volume, shape::Cuboid};

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use amethyst_rendy::sprite::SpriteRender;

use crate::{
    components::collider::Collider,
    entities::{enemy::{Enemy, summon_ghost}, laser::Laser},
    resources::audio::{SoundType, Sounds},
};

/// This is the main laser collision detection system, or LCDS.
/// Note: an alternative approach (probably more useful in larger games)
/// would be using ncollide's broad phase collision detection and integrating
/// it with amethyst. Then it would be tracking a whole lot of things and reporting
/// more data.
#[derive(SystemDesc)]
pub struct CollisionSystem;

impl<'s> System<'s> for CollisionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Laser>,
        WriteStorage<'s, Enemy>,
        Entities<'s>,
        ReadStorage<'s, Collider>,
        ReadStorage<'s, SpriteRender>,
        ReadExpect<'s, LazyUpdate>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (transforms, lasers, mut enemies, entities, colliders, sprite_renders, lazy_update, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (laser_entity, _laser_a, transform_a) in (&entities, &lasers, &transforms).join() {
            // the x, y should be the half length along the x and y axes, respectively
            // for a ball type you'd use a radius instead. this creates a representation of
            // the shape and a size of the shape, but *not* positioning of any kind
            // this number should be in a config somewhere... it's the pixel width 7 and height 1,
            // both scaled by 5, and then divided in two to get the half length
            let laser_cube = Cuboid::new(Vector2::new(17.5, 2.5));

            // next we need to create an isometry representation of the position, which for 2d
            // ncollide is a vector of the x and y coordinates and a rotation (zero() for no rotation).
            // the actual rotation is available via some_transform.isometry() if ever needed
            let laser_cube_pos = Isometry2::new(
                Vector2::new(transform_a.translation().x, transform_a.translation().y),
                nalgebra::zero(),
            );

            // a bounding volume is the combination of a shape and a position
            let aabb_laser = bounding_volume::aabb(&laser_cube, &laser_cube_pos);

            for (enemy_entity, enemy, enemy_transform, enemy_collider, sprite_render) in
                (&entities, &mut enemies, &transforms, &colliders, &sprite_renders).join()
            {
                let x = enemy_transform.translation().x;
                let y = enemy_transform.translation().y;

                let collides = enemy_collider.intersects(x, y, &aabb_laser);

                // we don't want lasers to hit an enemy that is dead, which is
                // possible if more than one laser hits in a frame
                if collides && !enemy.is_dead() {
                    enemy.take_damage(20.0);
                    // we should probably destroy the laser too
                    entities.delete(laser_entity).unwrap();
                    // if the enemy has taken enough damage, delete them
                    if enemy.is_dead() && entities.delete(enemy_entity).is_ok() {
                        //info!("enemy deleted due to insufficient laser dodging abilities");
                        summon_ghost(sprite_render.clone(), enemy_transform.clone(), &entities, &lazy_update);
                        sounds.play_sound(SoundType::EnemyDeath, &storage, audio_output.as_deref());
                    }
                }
            }
        }
    }
}
