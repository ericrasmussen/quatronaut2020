use std::{iter::Cycle, vec::IntoIter};

use amethyst::{
    assets::Loader,
    audio::{AudioSink, DjSystemDesc, OggFormat, SourceHandle},
    core::{bundle::SystemBundle, SystemDesc},
    ecs::{DispatcherBuilder, World, WorldExt},
    error::Error,
};

// starting out by copying the pong example in the amethyst book
const MUSIC_TRACKS: &[&str] = &["music/Quatronaut_-_Angles_Of_Attack_v01.ogg"];

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

/// TODO: this is duplicated in audio.rs
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

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

    // Add sound effects and music to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(music);
}

// the DJ system assumes the type of resource it needs will exist already, but in main.rs
// we haven't initialized anything. this bundle takes care of initialization and adding the
// system so that it can be used by main.rs
// in the pong example it only works because pong is bundled and initialized first
// alternatively maybe a different djsystem could be added to gameplay.rs using the lower
// level DjSystem API
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
