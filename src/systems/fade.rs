//! This system controllers `Fader` components to create a
//! fade-to-black effect for simple level transitions.
use amethyst::{
    core::timing::Time,
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, Write, WriteStorage},
    renderer::{palette::Srgba, resources::Tint},
};

use crate::components::fade::{FadeStatus, Fader};

/// Faders have an associated `Tint` component so we can modify
/// the alpha frame by frame.
#[derive(SystemDesc)]
pub struct FadeSystem;

impl<'s> System<'s> for FadeSystem {
    type SystemData = (
        WriteStorage<'s, Fader>,
        WriteStorage<'s, Tint>,
        Write<'s, FadeStatus>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut faders, mut tints, mut fade_status, time): Self::SystemData) {
        for (fader, tint) in (&mut faders, &mut tints).join() {
            //info!("found tint: {:?}", tint);
            let alpha = fader.next_alpha_change(time.delta_seconds());
            let next_tint = Srgba::new(0.0, 0.0, 0.0, alpha);
            tint.0 = next_tint;
            fade_status.update(fader.clone())
            //info!("setting alpha tint to {:?}", tint);
        }
    }
}
