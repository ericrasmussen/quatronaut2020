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

    // MENU REFACTOR: for everything described below that should be moved, the
    // `app_root` could be passed into the menu state, or specific configs, or
    // a new struct containing all the loaded config info

    // MENU REFACTOR: this can be loaded in main menu on the gameplay start event
    let level_config = app_root.join("config").join("levels.ron");
    let levels = resources::level::LevelConfig::load(&level_config).unwrap();
    let all_levels = resources::level::get_all_levels(levels);

    // MENU REFACTOR: any menu sounds can be loaded in the menu if needed,
    // but either way the main menu can pass this to gameplay too
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

    let starting_mode = states::GameplayMode::LevelMode;
    let mut game = Application::new(
        assets,
        // add level config here to a config struct of some kind
        states::MainMenu::new(all_levels, sounds, starting_mode, false),
        game_data,
    )?;
    game.run();

    Ok(())
}
