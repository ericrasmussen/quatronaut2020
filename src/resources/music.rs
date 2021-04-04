//! This is a pretty big file for looping one track over and over, but
//! it could be used to add more tracks in the future. It's adapted from
//! the pong example in the amethyst book.
use std::{iter::Cycle, vec::IntoIter};

use amethyst::{
    assets::Loader,
    audio::{AudioSink, DjSystemDesc, OggFormat, SourceHandle},
    core::{bundle::SystemBundle, SystemDesc},
    ecs::{DispatcherBuilder, World, WorldExt},
    error::Error,
};

const MUSIC_TRACKS: &[&str] = &["music/Quatronaut_-_Angles_Of_Attack_v01.ogg"];

/// Our struct only needs to know about cycling over some number of handles.
pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

/// This is duplicated in audio.rs, but for now music related setup is being
/// kept here in it's own module. Probably ok to copy/paste
/// until there's a third use case.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// This loads all the music (our one track) and music struct
/// into the `world`.
pub fn initialize_music(world: &mut World) {
    let music = {
        let loader = world.read_resource::<Loader>();

        let mut sink = world.write_resource::<AudioSink>();
        sink.set_volume(0.5);

        let music = MUSIC_TRACKS
            .iter()
            .map(|file| load_audio_track(&loader, &world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();

        Music { music }
    };

    world.insert(music);
}

/// The DJ system assumes the type of resource it needs will exist already, but in main.rs
/// we haven't initialized anything. this bundle takes care of initialization and adding the
/// system so that it can be used by `main.rs`.
/// Note: in the pong example it only works because pong is bundled and initialized first.
/// Alternatively, maybe a different djsystem could be added to gameplay.rs using the lower
/// level `DjSystem` API (instead of `DjSystemDesc`).
pub struct MusicBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MusicBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            DjSystemDesc::new(|music: &mut Music| music.music.next()).build(world),
            "dj_system",
            &[],
        );
        initialize_music(world);
        Ok(())
    }
}
