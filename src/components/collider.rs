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
/// requiring collision detection. For now it assumes 2d Cuboid shapes
/// and axis aligned bounding box checks.
/// The shape itself is dependent on sprite size and scale, so these
/// components can be part of the prefabs that contain sprite information.
pub struct Collider {
    pub half_width: f32,
    pub half_height: f32,
}

impl Collider {
    // this assumes no rotation, but we can get Euler angles from
    // an entity's Transform component if we need to check rotation too
    // maybe change this to `aabb_from_transform` to clean up the systems code too
    pub fn aabb_from_coordinates(self, x: f32, y: f32) -> AABB<f32> {
        let half_extents = Vector2::new(self.half_width, self.half_height);

        let cube = Cuboid::new(half_extents);

        let pos = Isometry2::new(Vector2::new(x, y), nalgebra::zero());
        bounding_volume::aabb(&cube, &pos)
    }

    // This API is a little clunky, but keeping it for now because storing
    // the vector coordinates in this struct would imply another system to
    // continually update them based on the associated transform.
    pub fn intersects(self, x: f32, y: f32, other: &AABB<f32>) -> bool {
        let this_aabb = self.aabb_from_coordinates(x, y);
        this_aabb.intersects(other)
    }
}

impl Component for Collider {
    type Storage = DenseVecStorage<Self>;
}
