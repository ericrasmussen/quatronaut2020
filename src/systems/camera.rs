use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{timing::Time, Transform},
    derive::SystemDesc,
    ecs::{Join, Read, ReadExpect, ReadStorage, System, SystemData, Write, WriteStorage},
};

use crate::{
    components::{cutscene::Cutscene, perspective::Perspective, tags::CameraTag},
    resources::audio::Sounds,
};

#[derive(SystemDesc)]
pub struct CameraShakeSystem;

impl<'s> System<'s> for CameraShakeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, CameraTag>,
        Write<'s, Perspective>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut transforms, cameras, mut perspective, time, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (transform, _camera) in (&mut transforms, &cameras).join() {
            // this uses prepend to keep shaking if we're not done shaking yet,
            // otherwise resets the z axis to 0 (unrotated)
            match perspective.next_z_rotation(time.delta_seconds()) {
                Some(next_z) => transform.prepend_rotation_z_axis(next_z),
                None => transform.set_rotation_z_axis(0.0),
            };

            // play a sound, if not played already
            if !perspective.sound_already_played() {
                sounds.play_sound(perspective.get_sound_type(), &storage, audio_output.as_deref());
                perspective.played_sound();
            }
        }
    }
}

#[derive(SystemDesc)]
pub struct CameraZoomSystem;

impl<'s> System<'s> for CameraZoomSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, CameraTag>,
        Write<'s, Cutscene>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut transforms, cameras, mut cutscene, time, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (transform, _camera) in (&mut transforms, &cameras).join() {
            // we also continue updating the scale as long as the `Cutscene` provides
            // Some(next_scale)
            let current_scale = transform.scale().x;
            if let Some(next_scale) = cutscene.next_scale(current_scale, time.delta_seconds()) {
                transform.set_scale(next_scale);
            }

            // play a sound, if not played already
            if !cutscene.sound_already_played() {
                sounds.play_sound(cutscene.get_sound_type(), &storage, audio_output.as_deref());
                cutscene.played_sound();
            }
        }
    }
}
