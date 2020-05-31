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

use crate::entities::{enemy::Enemy, laser::Laser};

//use log::info;

// big TODO: as this system gets more complicated, at some point it'll probably
// be worth using ncollide's broad phase collision
#[derive(SystemDesc)]
pub struct CollisionSystem;

impl<'s> System<'s> for CollisionSystem {
    type SystemData = (
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Laser>,
        WriteStorage<'s, Enemy>,
        Entities<'s>,
    );

    fn run(&mut self, (transforms, lasers, enemies, entities): Self::SystemData) {
        for (entity_a, _laser_a, transform_a) in (&entities, &lasers, &transforms).join() {
            /*
             * Initialize the shapes.
             */
            // this is for a laser much larger than ours. agh.
            // the x, y should be the half length along the x and y axes, respectively
            // for a ball type you'd use a radius instead. this creates a representation of
            // the shape and a size of the shape, but *not* positioning of any kind
            // this number should be in a config somewhere... it's the pixel width 7 and height 1,
            // both scaled by 5, and then divided in two to get the half length
            let laser_cube = Cuboid::new(Vector2::new(17.5, 2.5));
    
            // next we need to create an isometry representation of the position, which for 2d
            // ncollide is a vector of the x and y coordinates and a rotation (zero() for no rotation).
            // the actual rotation is available via some_transform.isometry(), but 
            let laser_cube_pos = Isometry2::new(
                Vector2::new(transform_a.translation().x, transform_a.translation().y),
                nalgebra::zero()
            );

            // a bounding volume is the combination of a shape and a position
            let aabb_cube1 = bounding_volume::aabb(&laser_cube, &laser_cube_pos);

            for (entity_b, _enemy_b, transform_b) in (&entities, &enemies, &transforms).join() {

                let enemy_cube = Cuboid::new(Vector2::new(70.0, 70.0));
                let enemy_cube_pos = Isometry2::new(
                    Vector2::new(transform_b.translation().x, transform_b.translation().y),
                    nalgebra::zero()
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
                    entities.delete(entity_b).unwrap();

                    // we should probably destroy the laser too
                    entities.delete(entity_a).unwrap();
                }
            }
        }
    }
}
