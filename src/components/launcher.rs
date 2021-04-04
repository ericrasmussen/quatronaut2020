//! This component tracks when and how to fire projectiles,
//! along with logic to create different projectiles.
use amethyst::{
    assets::PrefabData,
    core::Transform,
    derive::PrefabData,
    ecs::prelude::{Component, DenseVecStorage, Entities, Entity, LazyUpdate, NullStorage, ReadExpect, WriteStorage},
    renderer::{sprite::SpriteSheetHandle, SpriteRender},
    Error,
};

use rand::{thread_rng, Rng};

use serde::{Deserialize, Serialize};

use crate::components::{
    collider::Collider,
    movement::{Movement, MovementType},
    tags::CleanupTag,
};

use crate::resources::audio::SoundType;

/// This is used by the boss enemy that fires projectiles. The
/// launcher lets us control the firing rate and projectile speed.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Launcher {
    pub fire_delay: f32,
    pub projectile_speed: f32,
    pub seconds_since_firing: f32,
}

impl Launcher {
    /// Checks if we've had enough time elapse since the last laser
    /// and resets the timer. this is possibly a surprising API for a
    /// `bool` check, but it also ensures we don't rely on calling code
    /// to manage the timer.
    pub fn can_fire(&mut self, time: f32) -> bool {
        // this offset here is to make the firing less predictable,
        // which is important when multiple enemies would otherwise fire
        // each shot at the same time
        if self.seconds_since_firing >= self.fire_delay {
            let mut rng = thread_rng();
            self.seconds_since_firing = rng.gen_range(0.1..0.9);
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

/// Empty struct that lets us tag entities as `Projectile`s. The `systems`
/// module needs this for looking them up.
#[derive(Debug, Default)]
pub struct Projectile;

impl Component for Projectile {
    type Storage = NullStorage<Self>;
}

/// This needs to be run by a system that has a launcher, sprites, transforms,
/// and all entities. It creates an entity with all the necessary components
/// for systems to operate on the projectile (moving it, detecting collisions,
/// checking if it's out of bounds, etc).
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
        launch_sound: Some(SoundType::EnemyBlaster),
        movement_type: MovementType::ProjectileRush,
    };

    let collider = Collider {
        half_width: 16.0,
        half_height: 16.0,
    };

    let projectile = Projectile {};
    let cleanup_tag = CleanupTag {};

    let projectile_entity: Entity = entities.create();
    lazy_update.insert(projectile_entity, projectile);
    lazy_update.insert(projectile_entity, cleanup_tag);
    lazy_update.insert(projectile_entity, movement);
    lazy_update.insert(projectile_entity, transform);
    lazy_update.insert(projectile_entity, collider);
    lazy_update.insert(projectile_entity, sprite_render);
}
