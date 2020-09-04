use amethyst::{
    assets::PrefabData,
    derive::PrefabData,
    ecs::{storage::DenseVecStorage, Component, Entity, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

use crate::components::{collider::Collider, launcher::Launcher, movement::Movement};

//use log::info;

// this entity is a grouping of components, which allows the prefab loads to aggregate
// components from a config file (`prefabs/enemy.ron` in our case)
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

// this is the enemy component, which should go in a separate components/enemy.rs mod.
// the velocity is used for our tracking system, which isn't able to update transforms
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Enemy {
    pub health: f32,
}

impl Enemy {
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
