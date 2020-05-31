use nalgebra::{Isometry2, Vector2};
use ncollide2d::{
    bounding_volume::{self, BoundingVolume},
    shape::Cuboid,
};

use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, WriteStorage},
};

use crate::entities::{enemy::Enemy, player::Player};

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
        Entities<'s>,
    );

    // we don't need `player` here, though if we add health it'd be useful. keeping for now
    // until deciding
    fn run(&mut self, (transforms, players, enemies, entities): Self::SystemData) {
        for (player_entity, _player, player_transform) in (&entities, &players, &transforms).join() {
            /*
             * Initialize the shapes.
             */

            // see collision.rs for details on the calculations, and let us hope this code does
            // not live forever.
            // note that it should be 40.0, 57.5 for the player but we want to be a little
            // forgiving. it is the year 3000, after all
            let player_cube = Cuboid::new(Vector2::new(35.0, 52.5));

            // next we need to create an isometry representation of the position, which for 2d
            // ncollide is a vector of the x and y coordinates and a rotation (zero() for no rotation).
            // the actual rotation is available via some_transform.isometry(), but
            let player_cube_pos = Isometry2::new(
                Vector2::new(player_transform.translation().x, player_transform.translation().y),
                nalgebra::zero(),
            );

            // a bounding volume is the combination of a shape and a position
            let aabb_cube1 = bounding_volume::aabb(&player_cube, &player_cube_pos);

            for (_enemy_entity, _enemy, enemy_transform) in (&entities, &enemies, &transforms).join() {
                let enemy_cube = Cuboid::new(Vector2::new(70.0, 70.0));
                let enemy_cube_pos = Isometry2::new(
                    Vector2::new(enemy_transform.translation().x, enemy_transform.translation().y),
                    nalgebra::zero(),
                );

                /*
                 * Compute their axis-aligned bounding boxes.
                 */
                let aabb_cube2 = bounding_volume::aabb(&enemy_cube, &enemy_cube_pos);

                // optional logging to check coordinates
                //info!("aab1: {:?} AND aab2: {:?}", &aabb_cube1, &aabb_cube2);
                //info!("does it collide? {:?}", aabb_cube1.intersects(&aabb_cube2));
                if aabb_cube1.intersects(&aabb_cube2) {
                    // this should be a call to some enemy method for reducing health
                    entities.delete(player_entity);
                }
            }
        }
    }
}
