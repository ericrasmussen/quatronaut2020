/// The `Player` in our game is a robot, presumably named Benitron 3000.
/// The robot has its own movement speed and fire delay, the latter of which
/// is used to prevent the laser firing as fast as possible based on the
/// current framerate.
use amethyst_rendy::sprite::prefab::{SpriteRenderPrefab,SpriteSheetPrefab};
use amethyst::{
    assets::{
        PrefabData,
        ProgressCounter,
    },
    core::Transform,
    derive::PrefabData,
    ecs::{
        storage::DenseVecStorage, Component, WriteStorage,
        Entity,
    },
    Error,
};

use serde::{Deserialize, Serialize};
// this entity is a grouping of components, which allows the prefab loads to aggregate
// components from a config file (`prefabs/enemy.ron` in our case)
#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerPrefab {
    pub sheet: SpriteSheetPrefab,
    pub render: SpriteRenderPrefab,
    pub transform: Transform,
    pub player: Player,
}

impl<'a> PrefabData<'a> for PlayerPrefab {
    type SystemData = (
        <SpriteSheetPrefab as PrefabData<'a>>::SystemData,
        <SpriteRenderPrefab as PrefabData<'a>>::SystemData,
        <Transform as PrefabData<'a>>::SystemData,
        <Player as PrefabData<'a>>::SystemData,
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
        &self.transform.add_to_entity(entity, &mut system_data.2, entities, children)?;
        &self.player.add_to_entity(entity, &mut system_data.3, entities, children)?;
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub speed: f32,
    pub laser_speed: f32,
    // time to delay laser shots in seconds
    pub fire_delay: f32,
    pub seconds_since_firing: f32,
}

impl Player {
    // this is mainly so callers cannot modify the speed directly. we could
    // also have the player track momentum to compute a speed, but it seems
    // unnecessary
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

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

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}
