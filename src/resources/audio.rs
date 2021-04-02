use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, OggFormat, Source, SourceHandle},
    ecs::{World, WorldExt},
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SoundType {
    PlayerBlaster,
    PlayerDeath,
    EnemyBlaster,
    EnemyDeath,
    TriangleLock,
    ShortTransition,
    LongTransition,
    GlassTransition,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundConfig {
    max_volume: f32,
    player_blaster: Vec<String>,
    player_death: Vec<String>,
    enemy_blaster: Vec<String>,
    enemy_death: Vec<String>,
    triangle_lock: Vec<String>,
    short_transition: String,
    long_transition: String,
    glass_transition: String,
}

pub struct Sounds {
    pub volume: f32,
    pub player_blaster: Vec<SourceHandle>,
    pub enemy_blaster: Vec<SourceHandle>,
    pub enemy_death: Vec<SourceHandle>,
    pub player_death: Vec<SourceHandle>,
    pub triangle_lock: Vec<SourceHandle>,
    pub short_transition: SourceHandle,
    pub long_transition: SourceHandle,
    pub glass_transition: SourceHandle,
}

impl Sounds {
    pub fn random_int(&self, max: usize) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0, max)
    }

    pub fn play_sound(&self, sound_type: SoundType, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(ref output) = output.as_ref() {
            //info!("playing sound: {:?}", sound_type);
            let (volume, sound_ref) = match sound_type {
                SoundType::PlayerBlaster => {
                    let index = self.random_int(self.player_blaster.len() - 1);
                    (0.5, &self.player_blaster[index])
                },
                SoundType::PlayerDeath => {
                    let index = self.random_int(self.player_death.len() - 1);
                    (0.8, &self.player_death[index])
                },
                SoundType::EnemyBlaster => {
                    let index = self.random_int(self.enemy_blaster.len() - 1);
                    (0.5, &self.enemy_blaster[index])
                },
                SoundType::EnemyDeath => {
                    let index = self.random_int(self.enemy_death.len() - 1);
                    (0.6, &self.enemy_death[index])
                },
                SoundType::TriangleLock => {
                    let index = self.random_int(self.triangle_lock.len() - 1);
                    (0.7, &self.triangle_lock[index])
                },
                SoundType::ShortTransition => {
                    // we want the player to notice the crunching/shifting
                    (1.0, &self.short_transition)
                },
                SoundType::LongTransition => (1.0, &self.long_transition),
                SoundType::GlassTransition => (1.0, &self.glass_transition),
                SoundType::None => {
                    return;
                },
            };

            if let Some(sound) = storage.get(&sound_ref) {
                let balanced_volume = self.volume * volume;
                output.play_once(sound, balanced_volume);
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

        Sounds {
            volume: config.max_volume,
            player_blaster: config
                .player_blaster
                .iter()
                .map(|ogg| load_audio_track(&loader, &world, ogg))
                .collect(),
            enemy_blaster: config
                .enemy_blaster
                .iter()
                .map(|ogg| load_audio_track(&loader, &world, ogg))
                .collect(),
            player_death: config
                .player_death
                .iter()
                .map(|ogg| load_audio_track(&loader, &world, ogg))
                .collect(),
            enemy_death: config
                .player_death
                .iter()
                .map(|ogg| load_audio_track(&loader, &world, ogg))
                .collect(),
            triangle_lock: config
                .triangle_lock
                .iter()
                .map(|ogg| load_audio_track(&loader, &world, ogg))
                .collect(),
            short_transition: load_audio_track(&loader, &world, &config.short_transition),
            long_transition: load_audio_track(&loader, &world, &config.long_transition),
            glass_transition: load_audio_track(&loader, &world, &config.glass_transition),
        }
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}
