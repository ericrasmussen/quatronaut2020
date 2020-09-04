use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use crate::components::launcher::{launch_projectile, Launcher};

use amethyst_rendy::sprite::SpriteRender;

#[derive(SystemDesc)]
pub struct ProjectilesSystem;

// this system is likely too complicated, but it's not clear if there's a benefit
// to breaking some of it into separate systems (for instance, one system to track
// input, another to modify the transform, another to spawn lasers, etc)
#[allow(clippy::type_complexity)]
impl<'s> System<'s> for ProjectilesSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Launcher>,
        Entities<'s>,
        ReadStorage<'s, SpriteRender>,
        ReadExpect<'s, LazyUpdate>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut transforms, mut launchers, entities, sprites, lazy_update, time): Self::SystemData) {
        for (launcher, transform, sprite) in (&mut launchers, &mut transforms, &sprites).join() {
            if launcher.can_fire(time.delta_seconds()) {
                launch_projectile(
                    *launcher,
                    sprite.clone().sprite_sheet,
                    &transform,
                    &entities,
                    &lazy_update,
                );
            }
        }
    }
}
