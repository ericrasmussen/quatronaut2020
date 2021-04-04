//! The big red boss badguy has a `Launcher` component that can
//! fire projectiles. This module spawns those projectiles whenever
//! the boss can fire (as determined by their configured firing rate).
use amethyst::{
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, WriteStorage},
};

use crate::components::launcher::{launch_projectile, Launcher};

use amethyst_rendy::sprite::SpriteRender;

#[derive(SystemDesc)]
pub struct ProjectilesSystem;

/// Launch some projectiles whenever an enemy is ready to fire! This
/// uses `Launcher.can_fire` (which internally has a time-based firing rate)
/// so that enemies fire only periodically, and not once per frame. I did
/// accidentally let them fire once per frame though and it looked neat.
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
