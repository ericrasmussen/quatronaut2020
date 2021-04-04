//! Working on crate docs
// this is the main entry point for our game. it was only slightly modified
// from the main.rs file in https://github.com/amethyst/amethyst-starter-2d
use amethyst::{
    assets::PrefabLoaderSystemDesc,
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};

mod components;
mod entities;
mod resources;
mod states;
mod systems;
use entities::{enemy::EnemyPrefab, player::PlayerPrefab};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets = app_root.join("assets");
    let display_config = app_root.join("config").join("display_config.ron");
    let binding_path = app_root.join("config").join("bindings.ron");

    let level_config_path = app_root.join("config").join("levels.ron");
    let level_config = resources::level::LevelConfig::load(&level_config_path).unwrap();
    let all_levels = resources::level::get_all_levels(level_config.clone());

    let sound_config = app_root.join("config").join("audio.ron");
    let sounds = resources::audio::SoundConfig::load(&sound_config).unwrap();

    let input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(AudioBundle::default())?
        .with_system_desc(PrefabLoaderSystemDesc::<EnemyPrefab>::default(), "", &[])
        .with_system_desc(PrefabLoaderSystemDesc::<PlayerPrefab>::default(), "", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config)?.with_clear([0.0, 0.0, 0.0, 1.0]))
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?
        .with_bundle(resources::music::MusicBundle)?;

    let starting_mode = resources::gameconfig::GameplayMode::LevelMode;
    let game_config = resources::gameconfig::GameConfig {
        level_config,
        current_levels: all_levels,
        sound_config: sounds,
        gameplay_mode: starting_mode,
        immortal_hyper_mode: false,
    };
    let mut game = Application::new(
        assets,
        // add level config here to a config struct of some kind
        states::MainMenu::new(game_config, false),
        game_data,
    )?;
    game.run();

    Ok(())
}
