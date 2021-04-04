//! The `Collider` component can be attached to entities when
//! we need collision detection checks, e.g. in the `systems` module.
//! Currently it only supports 2d cuboid shapes and axis-aligned
//! bounding checks.
use nalgebra::{Isometry2, Vector2};
use ncollide2d::{
    bounding_volume::{self, BoundingVolume, AABB},
    shape::Cuboid,
};

use amethyst::{
    assets::PrefabData,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
/// Standalone Collider component that can be attached to any entities
/// requiring collision detection.
pub struct Collider {
    pub half_width: f32,
    pub half_height: f32,
}

impl Collider {
    /// This method takes the (x, y) coordinates of an entity (these
    /// should come from the `Transform` component on the entity) and
    /// creates an axis-aligned bounding box. Note that we assume
    /// there is no rotation (since there is no meaningful use for
    /// detecting collisions at different rotations in this game),
    /// but it is possible to get Euler angles from `Transform`s
    /// if ever needed.
    pub fn aabb_from_coordinates(self, x: f32, y: f32) -> AABB<f32> {
        let half_extents = Vector2::new(self.half_width, self.half_height);

        let cube = Cuboid::new(half_extents);

        let pos = Isometry2::new(Vector2::new(x, y), nalgebra::zero());
        bounding_volume::aabb(&cube, &pos)
    }

    /// This lets us check if the given (x, y) coordinates collide with
    /// another axis-aligned bounding box.
    /// This API is a little clunky, but keeping it for now because storing
    /// the vector coordinates in this struct would imply another system to
    /// continually update them based on the associated transform.
    pub fn intersects(self, x: f32, y: f32, other: &AABB<f32>) -> bool {
        let this_aabb = self.aabb_from_coordinates(x, y);
        this_aabb.intersects(other)
    }
}

impl Component for Collider {
    type Storage = DenseVecStorage<Self>;
}

// these tests might be better with generated data. right now it's
// hardly exhaustive, but at least serves to exercise the code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersects() {
        let player_collider = Collider {
            half_width: 28.0,
            half_height: 28.0,
        };
        let enemy_collider = Collider {
            half_width: 32.0,
            half_height: 32.0,
        };
        // the (x, y) coordinates of the player and enemy are both close enough
        // to be touching, given the half extents above
        let enemy_aabb = enemy_collider.aabb_from_coordinates(100.0, 100.0);
        assert_eq!(player_collider.intersects(120.0, 90.0, &enemy_aabb), true);
    }

    #[test]
    fn test_does_not_intersect() {
        let player_collider = Collider {
            half_width: 28.0,
            half_height: 28.0,
        };
        let enemy_collider = Collider {
            half_width: 32.0,
            half_height: 32.0,
        };
        let enemy_aabb = enemy_collider.aabb_from_coordinates(100.0, 100.0);
        assert_eq!(player_collider.intersects(500.0, 500.0, &enemy_aabb), false);
    }
}
