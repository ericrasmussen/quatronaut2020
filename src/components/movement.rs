//! The `Movement` component is a struct that keeps useful information
//! about how enemies should move. The systems module then uses this data
//! to update `Transform`s or other associated components.
//! This went through a lot of changes early on so it's kind of messy,
//! in the sense that the `Movement` struct captures optional data about
//! all kinds of potential movement, rather than having the different
//! movement types separated out.
use amethyst::{
    assets::PrefabData,
    core::math::Vector3,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

use crate::resources::audio::SoundType;

/// The supported types of enemy/projectile movement in our game.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MovementType {
    Gravitate,
    HorizontalRush,
    ProjectileRush,
}

impl Default for MovementType {
    fn default() -> Self {
        MovementType::Gravitate
    }
}

/// A struct that captures all sorts of data about all sorts of
/// movement. It grew too big and could probably use a redesign.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Movement {
    pub speed: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub freeze_direction: bool,
    pub locked_direction: Option<Vector3<f32>>,
    pub already_rotated: bool,
    pub launch_sound: Option<SoundType>,
    pub movement_type: MovementType,
}

impl Component for Movement {
    type Storage = DenseVecStorage<Self>;
}

impl Movement {
    /// this is mainly so callers cannot modify the speed directly
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// This takes incoming (x, y) data about the target, i.e. the player the enemy
    /// is chasing, and the enemy's current (x, y) coordinates, then decides what the
    /// updated (x, y) position should be based on the `MovementType`.
    pub fn next_move(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        match self.movement_type {
            MovementType::Gravitate => self.move_towards(target_x, target_y, current_x, current_y),
            MovementType::HorizontalRush => self.rush_towards(target_x, target_y, target_z, current_x, current_y),
            MovementType::ProjectileRush => self.projectile_rush(target_x, target_y, target_z, current_x, current_y),
        }
    }

    /// Standard enemy movement is to "gravitate" towards (slowly pursue) the
    /// player. This equation was quite painful to work out. The tl;dr is it
    /// computes the angle between the target/player's point and the enemy's
    /// own current point on a 2d plane. Then it updates the x and y velocity
    /// based on their speed. Note that it does not actually move the enemy.
    /// A separate system will handle movement based on velocity and delta time.
    pub fn move_towards(&mut self, target_x: f32, target_y: f32, current_x: f32, current_y: f32) {
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let angle = dy.atan2(dx);

        self.velocity_x = self.get_speed() * angle.cos();
        self.velocity_y = self.get_speed() * angle.sin();
    }

    /// This is a strategy used by the yellow triangle enemies to see if the player is within
    /// a certain range either horizontally or vertically. It then locks in its direction
    /// and velocity so that it will shoot out in one direction without turning (the movement
    /// system is responsible for updating the actual position based on this velocity
    /// and the locked direction).
    pub fn rush_towards(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        let player_in_range = (current_x - target_x).abs() <= 150.0 || (current_y - target_y).abs() <= 150.0;

        if !self.freeze_direction && player_in_range {
            // basic idea is that when the player is within a certain range, the enemy will
            // set off once in that direction only and not change
            self.move_towards(target_x, target_y, current_x, current_y);
            self.locked_direction = Some(Vector3::new(target_x, target_y, target_z));
            self.freeze_direction = true;
        }
    }

    /// This is the same idea as `rush_towards`, except with projectiles we don't want them
    /// to wait for the player to approach. They lock in their direction and velocity and
    /// start going immediately.
    pub fn projectile_rush(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        if !self.freeze_direction {
            self.locked_direction = Some(Vector3::new(target_x, target_y, target_z));
            self.freeze_direction = true;
            self.move_towards(target_x, target_y, current_x, current_y);
        }
    }
}
