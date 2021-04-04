//! This module contains the prefab setup and structs for our
//! hero, the brave blue circle that never gives up and never surrenders.
use amethyst::{
    assets::PrefabData,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

use crate::{components::collider::Collider, resources::direction::Direction};

/// This entity is a grouping of components, which allows the prefab loads to aggregate
/// components from a config file (`assets/prefabs/player.ron` in our case).
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerPrefab {
    pub player: Player,
    pub player_collider: Collider,
}

impl<'a> PrefabData<'a> for PlayerPrefab {
    type Result = ();
    type SystemData = (
        <Player as PrefabData<'a>>::SystemData,
        <Collider as PrefabData<'a>>::SystemData,
    );

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        self.player
            .add_to_entity(entity, &mut system_data.0, entities, children)?;
        self.player_collider
            .add_to_entity(entity, &mut system_data.1, entities, children)?;
        Ok(())
    }
}

/// This is the main struct that represents what it means to be
/// a true player for real (TPFR). We need to know the player's speed,
/// the speed of the lasers they fire, their fire delay (which determines
/// fire rate), their current direction, and whether or not they are an
/// immortal being impervious to all known forms of damage.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub speed: f32,
    pub laser_speed: f32,
    // time to delay laser shots in seconds
    pub fire_delay: f32,
    pub seconds_since_firing: f32,
    pub direction: Direction,
    pub invulnerable: bool,
}

impl Player {
    /// This is mainly so callers cannot modify the speed directly, but can
    /// still get the current speed.
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// Checks if we've had enough time elapse since the last laser
    /// and resets the timer. this is possibly a surprising API for a
    /// `bool` check, but it also ensures we don't rely on calling code
    /// to manage the timer.
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

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}
