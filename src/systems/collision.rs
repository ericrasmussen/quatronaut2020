use nalgebra::{Isometry2, Vector2};
use ncollide2d::{
    bounding_volume,
    shape::Cuboid,
};

use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, WriteStorage},
};

use crate::entities::{enemy::Enemy, laser::Laser};
use crate::components::collider::Collider;
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
        ReadStorage<'s, Collider>,
    );

    fn run(&mut self, (transforms, lasers, enemies, entities, colliders): Self::SystemData) {
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

            for (enemy_entity, _enemy, enemy_transform, enemy_collider) in (&entities, &enemies, &transforms, &colliders).join() {

                let collides = enemy_collider.intersects(
                    enemy_transform.translation().x,
                    enemy_transform.translation().y,
                    &aabb_cube1,
                );

                if collides {
                    entities.delete(enemy_entity).unwrap();
                    // we should probably destroy the laser too
                    entities.delete(entity_a).unwrap();
                }
            }
        }
    }
}
