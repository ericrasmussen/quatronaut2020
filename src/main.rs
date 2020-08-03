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
mod level;
mod state;
mod systems;
use entities::{enemy::EnemyPrefab, player::PlayerPrefab};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("resources");
    let display_config = app_root.join("config").join("display_config.ron");
    let binding_path = app_root.join("config").join("bindings.ron");

    // load all the levels
    let level_config = app_root.join("config").join("levels.ron");
    let levels = level::LevelConfig::load(&level_config).unwrap();
    let all_levels = level::get_all_levels(levels);

    let input_bundle = InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_system_desc(PrefabLoaderSystemDesc::<EnemyPrefab>::default(), "", &[])
        .with_system_desc(PrefabLoaderSystemDesc::<PlayerPrefab>::default(), "", &[])
        .with(systems::PlayerSystem, "player_system", &["input_system"])
        .with(systems::LaserSystem, "laser_system", &[])
        .with(systems::CollisionSystem, "collision_system", &[])
        .with(systems::AttackedSystem, "attacked_system", &[])
        .with(systems::EnemyTrackingSystem, "enemy_tracking_system", &[])
        .with(systems::EnemyMoveSystem, "enemy_move_system", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config)?.with_clear([255.0, 255.0, 255.0, 1.0]))
                .with_plugin(RenderFlat2D::default()),
        )?;

    // 2.0 is the seconds to delay before enemy wave 1 spawns
    let mut game = Application::new(
        resources,
        // the 10 here is a big hack. need to fix up enemy counting
        state::GameplayState::new(10, all_levels, true),
        game_data,
    )?;
    game.run();

    Ok(())
}
