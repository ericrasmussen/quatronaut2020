use nalgebra::{Isometry2, Vector2};
use ncollide2d::{bounding_volume, shape::Cuboid};

use amethyst::{
    core::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, Write, WriteStorage},
};

use crate::{
    components::collider::Collider,
    entities::{enemy::Enemy, laser::Laser},
    state::EnemyCount,
};

use log::info;

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
        Write<'s, EnemyCount>,
    );

    fn run(&mut self, (transforms, lasers, mut enemies, entities, colliders, mut enemy_count): Self::SystemData) {
        for (laser_entity, _laser_a, transform_a) in (&entities, &lasers, &transforms).join() {
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
                nalgebra::zero(),
            );

            // a bounding volume is the combination of a shape and a position
            let aabb_laser = bounding_volume::aabb(&laser_cube, &laser_cube_pos);

            // TODO: have an out of bounds check here too
            for (enemy_entity, enemy, enemy_transform, enemy_collider) in
                (&entities, &mut enemies, &transforms, &colliders).join()
            {
                let x = enemy_transform.translation().x;
                let y = enemy_transform.translation().y;

                let collides = enemy_collider.intersects(x, y, &aabb_laser);

                // these values should be based on game dimensions. the check is needed
                // for enemies that move off screen before getting hit
                let out_of_bounds = x < -500.0 || x > 2500.0 || y < -500.0 || y > 2500.0;

                if collides {
                    enemy.take_damage(20.0);
                    // we should probably destroy the laser too
                    entities.delete(laser_entity).unwrap();
                }

                // if the enemy has taken enough damage or is out of bounds, delete them
                if enemy.is_dead() || out_of_bounds {
                    entities.delete(enemy_entity).unwrap();
                    enemy_count.decrement_by(1);
                    info!("enemy deleted!!!!!");
                }
            }
        }
    }
}
