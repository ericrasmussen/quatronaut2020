//! The enemy entities are the main antagonists in the
//! game. They have a lot of associated data, like their
//! movement type, whether they can fire projectiles, their
//! health, a transform, a collider, etc. To manage all these I began
//! originally by using prefabs. I found them to be somewhat inflexible
//! (although I might be using them wrong), so below is a hybrid approach.
//! The data that fit in a config file can be found under assets/prefabs,
//! and the rest (like transforms), that depend on runtime decisions,
//! are decided by `gameplay.rs`.

use amethyst::{
    assets::PrefabData,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

use crate::components::{collider::Collider, launcher::Launcher, movement::Movement};

// This entity is a grouping of components representing one game enemy,
// which allows the prefab loads to aggregate components from a config
// file (e.g. `assets/prefabs/enemy.ron`).
#[derive(Debug, Deserialize, Serialize)]
pub struct EnemyPrefab {
    pub enemy: Enemy,
    pub collider: Collider,
    pub movement: Movement,
    pub launcher: Option<Launcher>,
}

impl<'a> PrefabData<'a> for EnemyPrefab {
    type Result = ();
    type SystemData = (
        <Enemy as PrefabData<'a>>::SystemData,
        <Collider as PrefabData<'a>>::SystemData,
        <Movement as PrefabData<'a>>::SystemData,
        <Launcher as PrefabData<'a>>::SystemData,
    );

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        self.enemy
            .add_to_entity(entity, &mut system_data.0, entities, children)?;
        self.collider
            .add_to_entity(entity, &mut system_data.1, entities, children)?;
        self.movement
            .add_to_entity(entity, &mut system_data.2, entities, children)?;
        self.launcher
            .add_to_entity(entity, &mut system_data.3, entities, children)?;
        Ok(())
    }
}

/// This probably belongs under components/, but the definition of an entity
/// is somewhat loose when using prefabs, because by definition they are
/// many associated components. The enemy struct itself is mostly just for
/// figuring our whether the enemy died yet, and reducing their health.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Enemy {
    pub health: f32,
}

impl Enemy {
    /// Has the enemy died? Can polygons truly die? A question for the ages.
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// Use this in systems deciding when an enemy has taken some amount of
    /// damage, likely from the player's laser weapon.
    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
