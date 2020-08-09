// this is the main entry point for our game. it was only slightly modified
// from the main.rs file in https://github.com/amethyst/amethyst-starter-2d
use amethyst::{
    assets::PrefabLoaderSystemDesc,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
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

    // load all the levels
    let level_config = app_root.join("config").join("levels.ron");
    let levels = resources::level::LevelConfig::load(&level_config).unwrap();
    let all_levels = resources::level::get_all_levels(levels);

    let input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_system_desc(PrefabLoaderSystemDesc::<EnemyPrefab>::default(), "", &[])
        .with_system_desc(PrefabLoaderSystemDesc::<PlayerPrefab>::default(), "", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config)?.with_clear([0.0, 0.0, 0.0, 1.0]))
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets, states::GameplayState::new(all_levels), game_data)?;
    game.run();

    Ok(())
}
