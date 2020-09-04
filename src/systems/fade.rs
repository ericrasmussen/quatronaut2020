use amethyst::{
    core::timing::Time,
    derive::SystemDesc,
    ecs::{Join, Read, System, SystemData, Write, WriteStorage},
    renderer::{palette::Srgba, resources::Tint},
};

use crate::resources::fade::{FadeStatus, Fader};

//use log::info;

#[derive(SystemDesc)]
pub struct FadeSystem;

// this system exists to remove the player entity when victory is achieved
// if it happens too suddenly we might do something like a victory condition
// resource that requires both enemy count <= 0 and some time elapsed since
// the last enemy was removed
impl<'s> System<'s> for FadeSystem {
    type SystemData = (
        WriteStorage<'s, Fader>,
        WriteStorage<'s, Tint>,
        Write<'s, FadeStatus>,
        Read<'s, Time>,
    );

    // open question: should lasers be deleted here too? otherwise they may still be flying
    // across the screen when a new level starts
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
