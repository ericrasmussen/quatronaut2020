/// This component tracks when and how to fire projectiles,
/// along with logic to create different projectiles.

use amethyst::{
    assets::PrefabData,
    core::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, ReadExpect, WriteStorage},
    derive::PrefabData,
    renderer::{sprite::SpriteSheetHandle, SpriteRender},
    Error,
};

use serde::{Deserialize, Serialize};

use crate::components::movement::{Movement, MovementType};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Launcher {
    pub fire_delay: f32,
    pub projectile_speed: f32,
    pub seconds_since_firing: f32,
}

impl Launcher {
    // checks if we've had enough time elapse since the last laser
    // and resets the timer. this is possibly a surprising API for a
    // `bool` check, but it also ensures we don't rely on calling code
    // to manage the timer.
    pub fn can_fire(&mut self, time: f32) -> bool {
        if self.seconds_since_firing >= self.fire_delay {
            self.seconds_since_firing = 0.0;
            true
        } else {
            self.seconds_since_firing += time;
            false
        }
    }
}

impl Component for Launcher {
    type Storage = DenseVecStorage<Self>;
}

// this needs to be run by a system that has a launcher, sprites, transforms,
// and all entities.
pub fn launch_projectile(
    launcher: Launcher,
    sprite_sheet_handle: SpriteSheetHandle,
    base_transform: &Transform,
    entities: &Entities,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    // an incorrect sprite number here will lead to a memory leak
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 3,
    };

    let transform = base_transform.clone();

    let movement = Movement {
        speed: launcher.projectile_speed,
        velocity_x: 0.0,
        velocity_y: 0.0,
        freeze_direction: false,
        locked_direction: None,
        already_rotated: false,
        movement_type: MovementType::HorizontalRush,
    };

    let projectile_entity: Entity = entities.create();
    lazy_update.insert(projectile_entity, movement);
    lazy_update.insert(projectile_entity, transform);
    lazy_update.insert(projectile_entity, sprite_render);
}
