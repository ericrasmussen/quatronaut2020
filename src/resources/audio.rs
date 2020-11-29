use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, OggFormat, Source, SourceHandle},
    ecs::{World, WorldExt},
};
//use log::info;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundConfig {
    volume: f32,
    player_blaster: Vec<String>,
    player_death: Vec<String>,
    enemy_blaster: Vec<String>,
    enemy_death: Vec<String>,
    triangle_lock: String,
    transition: String,
}

#[derive(Debug, Clone)]
pub enum SoundType {
    PlayerBlaster,
    PlayerDeath,
    //EnemyBlaster,
    EnemyDeath,
    TriangleLock,
}

pub struct Sounds {
    pub volume: f32,
    pub player_blaster: Vec<SourceHandle>,
    pub enemy_blaster: Vec<SourceHandle>,
    pub enemy_death: Vec<SourceHandle>,
    pub player_death: Vec<SourceHandle>,
    pub triangle_lock: SourceHandle,
}

impl Sounds {

    pub fn random_int(&self, max: usize) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0, max)
    }

    pub fn play_sound(&self, sound_type: SoundType, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(ref output) = output.as_ref() {
            let sound_ref = match sound_type {
                SoundType::PlayerBlaster => {
                    let index = self.random_int(self.player_blaster.len() - 1);
                    &self.player_blaster[index]
                },
                SoundType::PlayerDeath => {
                    let index = self.random_int(self.player_death.len() - 1);
                    &self.player_death[index]
                },
                // SoundType::EnemyBlaster => {
                //     let index = self.random_int(self.enemy_blaster.len() - 1);
                //     &self.enemy_blaster[index]
                // },
                SoundType::EnemyDeath => {
                    let index = self.random_int(self.enemy_death.len() - 1);
                    &self.enemy_death[index]
                },
                SoundType::TriangleLock => &self.triangle_lock,
            };

            if let Some(sound) = storage.get(&sound_ref) {
                output.play_once(sound, self.volume);
            }
        }
    }
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world. This will eventually include
/// the background tracks as well as the sound effects, but for now
/// we'll just work on sound effects.
pub fn initialize_audio(world: &mut World, config: &SoundConfig) {
    let sound_effects = {
        let loader = world.read_resource::<Loader>();

        let sound = Sounds {
            volume: config.volume,
            player_blaster: config.player_blaster
                .iter()
                .map(|ogg|load_audio_track(&loader, &world, ogg))
                .collect(),
            enemy_blaster: config.enemy_blaster
                .iter()
                .map(|ogg|load_audio_track(&loader, &world, ogg))
                .collect(),
            player_death: config.player_death
                .iter()
                .map(|ogg|load_audio_track(&loader, &world, ogg))
                .collect(),
            enemy_death: config.player_death
                .iter()
                .map(|ogg|load_audio_track(&loader, &world, ogg))
                .collect(),
            triangle_lock: load_audio_track(&loader, &world, &config.triangle_lock),
        };

        sound
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}
