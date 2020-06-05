/// An existential horror brought to life by the complexities of ECS.
//use amethyst::{
//    ecs::prelude::{Component, DenseVecStorage},
//    derive::PrefabData,
//    assets::PrefabData
//};

use amethyst_rendy::sprite::prefab::{SpriteRenderPrefab,SpriteSheetPrefab};
use amethyst::{
    assets::{
        PrefabData,
        ProgressCounter,
    },
    derive::PrefabData,
    ecs::{
        storage::DenseVecStorage, Component, Entity,
        WriteStorage,
    },
    Error,
};

use serde::{Deserialize, Serialize};

use crate::components::collider::Collider;

//use log::info;

// this entity is a grouping of components, which allows the prefab loads to aggregate
// components from a config file (`prefabs/enemy.ron` in our case)
#[derive(Debug, Deserialize, Serialize)]
pub struct EnemyPrefab {
    pub sheet: SpriteSheetPrefab,
    pub render: SpriteRenderPrefab,
    pub enemy: Enemy,
    pub collider: Collider,
}

impl<'a> PrefabData<'a> for EnemyPrefab {
    type SystemData = (
        <SpriteSheetPrefab as PrefabData<'a>>::SystemData,
        <SpriteRenderPrefab as PrefabData<'a>>::SystemData,
        <Enemy as PrefabData<'a>>::SystemData,
        <Collider as PrefabData<'a>>::SystemData,
    );

    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        &self.render.add_to_entity(entity, &mut system_data.1, entities, children)?;
        &self.enemy.add_to_entity(entity, &mut system_data.2, entities, children)?;
        &self.collider.add_to_entity(entity, &mut system_data.3, entities, children)?;
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let mut ret = false;
        if self.sheet.load_sub_assets(progress, &mut system_data.0)? {
                ret = true;
            }
        self.render.load_sub_assets(progress, &mut system_data.1)?;

        Ok(ret)
    }
}

// this is the enemy component, which should go in a separate components/enemy.rs mod.
// the velocity is used for our tracking system, which isn't able to update transforms
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Enemy {
    pub speed: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

impl Enemy {
    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    // probably doesn't belong here but since only enemies need this for now,
    // here's a function to compute how to move towards another transform
    // based on speed
    pub fn move_towards(&mut self, target_x: f32, target_y: f32, current_x: f32, current_y: f32) {
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let angle = dy.atan2(dx);

        self.velocity_x = &self.get_speed() * angle.cos();
        self.velocity_y = &self.get_speed() * angle.sin();
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
