use amethyst::{
    assets::PrefabData,
    core::math::Vector3,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MovementType {
    Gravitate,
    HorizontalRush,
    ProjectileRush,
    //PlayerControl,
}

impl Default for MovementType {
    fn default() -> Self {
        MovementType::Gravitate
    }
}

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
    pub movement_type: MovementType,
}

impl Component for Movement {
    type Storage = DenseVecStorage<Self>;
}

impl Movement {
    // what changes do we need here based on current usage? does it change?
    // enemy.next_move
    // if we just want to track speed + velocity then this could be used with
    // a system to update the transform.
    // but then enemies and bosses still need some shared component for
    // choosing the next velocity... agh.

    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    pub fn next_move(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        match self.movement_type {
            MovementType::Gravitate => self.move_towards(target_x, target_y, current_x, current_y),
            MovementType::HorizontalRush => self.rush_towards(target_x, target_y, target_z, current_x, current_y),
            MovementType::ProjectileRush => self.projectile_rush(target_x, target_y, target_z, current_x, current_y),
        }
    }

    // probably doesn't belong here but since only enemies need this for now,
    // here's a function to compute how to move towards another transform
    // based on speed
    pub fn move_towards(&mut self, target_x: f32, target_y: f32, current_x: f32, current_y: f32) {
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let angle = dy.atan2(dx);

        self.velocity_x = self.get_speed() * angle.cos();
        self.velocity_y = self.get_speed() * angle.sin();
    }

    // the rush strategy should be for picking one direction and then rushing
    pub fn rush_towards(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        let player_in_range = (current_x - target_x).abs() <= 150.0 || (current_y - target_y).abs() <= 150.0;

        if !self.freeze_direction && player_in_range {
            // kind of hacky, trying to see what works
            // the idea is that when the player is within a certain range, the enemy will
            // set off once in that direction only and not change
            self.move_towards(target_x, target_y, current_x, current_y);
            self.locked_direction = Some(Vector3::new(target_x, target_y, target_z));
            self.freeze_direction = true;
        }
    }

    // this is basically the rush strategy except it doesn't wait for the player to get close
    pub fn projectile_rush(&mut self, target_x: f32, target_y: f32, target_z: f32, current_x: f32, current_y: f32) {
        if !self.freeze_direction {
            self.locked_direction = Some(Vector3::new(target_x, target_y, target_z));
            self.freeze_direction = true;
            self.move_towards(target_x, target_y, current_x, current_y);
        }
    }
}
