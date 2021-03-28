/// This module contains our main gameplay state and game update method. It is
/// used by `main.rs` to build the application.
/// The main responsibilities are:
///   1) initialize the game world (assets, prefabs, entities)
///   2) setup the dispatcher so the systems here won't run in other states
///   3) act as the game's state manager (deciding when to switch states)
use amethyst::{
    assets::{Handle, PrefabLoader, ProgressCounter, RonFormat},
    core::math::{Translation3, UnitQuaternion, Vector3},
    core::{transform::Transform, ArcThreadPool},
    ecs::prelude::{Dispatcher, DispatcherBuilder, Entity, Join},
    ecs::world::EntitiesRes,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
    window::ScreenDimensions,
};

use derive_new::new;

use log::info;

use crate::entities::{
    enemy::{Enemy, EnemyPrefab},
    laser::Laser,
    player::{Player, PlayerPrefab},
};

use crate::{
    components::{
        collider::Collider,
        launcher::Launcher,
        movement::Movement,
        perspective::Perspective,
        tags::{BackgroundTag, CameraTag, CleanupTag},
    },
    resources::{
        audio,
        gameconfig::{GameConfig, GameplayMode},
        handles,
        handles::GameplayHandles,
        level::{EntityType, LevelMetadata, LevelStatus},
        music,
        playablearea::PlayableArea,
    },
    states::{alldone::AllDone, menu::MainMenu, paused::PausedState, transition::TransitionState},
    systems,
};

/// Collects our state-specific dispatcher, progress counter for asset
/// loading, struct with gameplay handles, and levels. Note that the
/// levels are loaded via `main.rs` (since they can be created from a
/// config file without gameplay state knowledge)
#[derive(new)]
pub struct GameplayState<'a, 'b> {
    // this is mostly for use in creating a new menu, but
    // there might be better options (like Trans::Pop)
    pub game_config: GameConfig,

    // default initializes this value with false
    #[new(default)]
    pub level_is_loaded: bool,

    #[new(default)]
    pub handles: Option<GameplayHandles>,

    #[new(default)]
    pub progress_counter: ProgressCounter,

    #[new(default)]
    pub dispatcher: Option<Dispatcher<'a, 'b>>,

    #[new(default)]
    pub high_score_text: Option<Entity>,
}

impl<'a, 'b> SimpleState for GameplayState<'a, 'b> {
    // runs once each time the program initializes a new `GameplayState`
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // creates a dispatcher to collect systems specific to this state
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(systems::PlayerSystem, "player_system", &[]);
        dispatcher_builder.add(systems::LaserSystem, "laser_system", &[]);
        dispatcher_builder.add(systems::CollisionSystem, "collision_system", &[]);
        dispatcher_builder.add(systems::AttackedSystem, "attacked_system", &[]);
        dispatcher_builder.add(systems::ProjectileHitSystem, "projectile_hit_system", &[]);
        dispatcher_builder.add(systems::MovementTrackingSystem, "movement_tracking_system", &[]);
        dispatcher_builder.add(systems::TransformUpdateSystem, "transform_update_system", &[]);
        dispatcher_builder.add(systems::ProjectilesSystem, "projectiles_system", &[]);

        // builds and sets up the dispatcher
        let mut dispatcher = dispatcher_builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();
        //info!("computed dimensions are: {:?}", &dimensions);

        // register our entities and resources before inserting them or
        // having them created as part of `init_level` in `update`
        world.register::<BackgroundTag>();
        world.register::<CameraTag>();
        world.register::<CleanupTag>();
        world.register::<Player>();
        world.register::<Laser>();
        world.register::<Enemy>();
        world.register::<Collider>();
        world.register::<Movement>();
        world.register::<Launcher>();
        world.register::<PlayableArea>();

        // Place the camera
        init_camera(world, &dimensions);

        // easier to load the prefab handles here and then pass them to
        let enemy_prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load("prefabs/enemy.ron", RonFormat, &mut self.progress_counter)
        });

        let flying_enemy_prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load("prefabs/flying_enemy.ron", RonFormat, &mut self.progress_counter)
        });

        let player_prefab_handle = world.exec(|loader: PrefabLoader<'_, PlayerPrefab>| {
            loader.load("prefabs/player.ron", RonFormat, &mut self.progress_counter)
        });

        let boss_prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load("prefabs/boss.ron", RonFormat, &mut self.progress_counter)
        });

        // load the remaining sprite sheets and collect all the handles used by `level_init`
        let gameplay_handles = handles::get_game_handles(
            world,
            &mut self.progress_counter,
            enemy_prefab_handle,
            flying_enemy_prefab_handle,
            player_prefab_handle,
            boss_prefab_handle,
        );
        self.handles = Some(gameplay_handles);

        // render the background
        init_background(
            world,
            &dimensions,
            self.handles.clone().unwrap().background_sprite_handle,
        );

        // audio should not need to be initialized multiple times
        audio::initialize_audio(world, &self.game_config.sound_config);

        // setup our music player
        music::initialize_music(world);

        // this will be used to match the type of level (if there are levels yet)
        // and other level metadata
        let next_level_status = self.game_config.current_levels.pop();

        // setup the playable area. this is still messy but if we begin in small level mode,
        // we setup the constrained level mode
        let playable_area = match next_level_status {
            LevelStatus::SmallLevel(_) => PlayableArea::new(dimensions.width(), dimensions.height(), true),
            _ => PlayableArea::new(dimensions.width(), dimensions.height(), false),
        };

        world.insert(playable_area);

        // switches back to the small background if needed
        change_background(world, &next_level_status);

        let handles = self.handles.clone().expect("failure accessing GameplayHandles struct");

        if let GameplayMode::LevelMode = &self.game_config.gameplay_mode {
            match next_level_status {
                LevelStatus::SmallLevel(next_level_metadata) => init_level(world, next_level_metadata, handles),
                LevelStatus::LargeLevel(next_level_metadata) => init_level(world, next_level_metadata, handles),
                LevelStatus::TransitionTime => self.game_config.gameplay_mode = GameplayMode::TransitionMode,
                LevelStatus::AllDone => self.game_config.gameplay_mode = GameplayMode::CompletedMode,
            };
        };
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            // TODO: we could maybe push another state over `GameplayState` and push back
            // when the counter is complete, rather than checking every time here
            if self.progress_counter.is_complete() {
                dispatcher.dispatch(&data.world);
            }
        }

        // since we can have potentially more than one player, this counts
        // them and lets us treat 0 `player_lives` as game over
        let player_lives = {
            let entities = data.world.read_resource::<EntitiesRes>();
            let enemies = data.world.read_storage::<Player>();
            (&entities, &enemies).join().count()
        };

        // this removes the need to track a count of enemies and have multiple
        // systems read and write to that resource
        let total = {
            let entities = data.world.read_resource::<EntitiesRes>();
            let enemies = data.world.read_storage::<Enemy>();
            (&entities, &enemies).join().count()
        };

        // still hacky. checks that we have at least 1 enemy to decide
        // that the level is loaded. this is because the levels inserted
        // into the world may not be loaded the first time this update is called
        if total > 0 {
            self.level_is_loaded = true;
        }

        let handles = self.handles.clone().expect("failure accessing GameplayHandles struct");

        // this branch decides whether or not to switch state. if a level is
        // loaded and all enemies are defeated, it's time to transition, otherwise
        // keep going
        if self.game_config.gameplay_mode == GameplayMode::TransitionMode {
            Trans::Switch(Box::new(TransitionState::new(
                handles.overlay_sprite_handle,
                self.game_config.clone(),
                Some(Perspective::new(1.8, 0.3, 3.0, audio::SoundType::LongTransition)),
            )))
        // we're in a level and all enemies are defeated -- fade out to a new level
        } else if total == 0 && self.level_is_loaded {
            Trans::Switch(Box::new(TransitionState::new(
                handles.overlay_sprite_handle,
                self.game_config.clone(),
                None,
            )))
        // we've finished the game! you did it! you're awesome! make sure this
        // comes before the game over check, because technically there are 0 players
        // when the game is complete
        } else if self.game_config.gameplay_mode == GameplayMode::CompletedMode {
            info!("YOU WIN!");
            Trans::Replace(Box::new(AllDone::new(self.game_config.clone(), true)))
        // the level is still going and you ran out of lives. keep tryin'
        } else if self.level_is_loaded && player_lives == 0 {
            info!("YOU LOSE!");
            Trans::Replace(Box::new(AllDone::new(self.game_config.clone(), false)))
        } else {
            Trans::None
        }
    }

    // handles pausing (toggling the `p` key) and closing (window close or pressing escape)
    fn handle_event(&mut self, _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Push(Box::new(MainMenu::new(self.game_config.clone(), true)));
            }

            if is_key_down(&event, VirtualKeyCode::P) {
                return Trans::Push(Box::new(PausedState));
            }
        }
        // no state changes required
        Trans::None
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // state items that should be cleaned up (players, entities, lasers,
        // projectiles) should all be marked with `CleanupTag` and removed
        // here when this state ends
        // note the separate scope because we're borrowing `data.world`
        // as immutable
        {
            let entities = data.world.read_resource::<EntitiesRes>();
            let cleanup_tags = data.world.read_storage::<CleanupTag>();

            for (entity, _tag) in (&entities, &cleanup_tags).join() {
                let err = format!("unable to delete entity: {:?}", entity);
                entities.delete(entity).expect(&err);
            }
        }
    }
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);
    info!(
        "computed dimensions: {:?} x {:?}",
        dimensions.width(),
        dimensions.height()
    );
    // many amethyst examples show using dimensions here, but it turns out we want the
    // intended dimensions (say, based on sprite sizes) and not the computed dimensions
    // (which are affected by hidpi and other factors, and may not be what we intended)
    // even though transforms can use the computed dimensions, the camera needs the original
    // dimensions (from display_config.ron)
    world
        .create_entity()
        .with(Camera::standard_2d(1920.0, 1080.0))
        .with(transform)
        .with(CameraTag::default())
        .build();
}

// render the background, giving it a low z value so it renders under
// everything else
fn init_background(world: &mut World, dimensions: &ScreenDimensions, bg_sprite_sheet_handle: Handle<SpriteSheet>) {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    let scale = Vector3::new(1.0, 1.0, 1.0);
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, -25.0);
    let transform = Transform::new(position, rotation, scale);

    // 0 here refers to the arcade machine background with a smaller playable window,
    // and 1 (which is used by `transition.rs`) is the widescreen background
    let sprite_number = 0;
    let bg_render = SpriteRender {
        sprite_sheet: bg_sprite_sheet_handle,
        sprite_number,
    };

    let bg_tag = BackgroundTag::default();

    world
        .create_entity()
        .with(bg_render)
        .with(bg_tag)
        .with(transform)
        .build();
}

fn change_background(world: &mut World, level_status: &LevelStatus) {
    // this is duplicated from transition.rs and should probably be
    // handled elsewhere. it resets the main background (from big to small)
    // for new games
    let sprite_number = match level_status {
        LevelStatus::LargeLevel(_) => 1,
        _ => 0,
    };

    let mut sprites = world.write_storage::<SpriteRender>();
    let backgrounds = world.read_storage::<BackgroundTag>();
    for (sprite, _bg) in (&mut sprites, &backgrounds).join() {
        sprite.sprite_number = sprite_number;
    }
}
fn init_level(world: &mut World, level_metadata: LevelMetadata, handles: GameplayHandles) {
    let playable_area = (*world.read_resource::<PlayableArea>()).clone();

    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(0.25, 0.25, 0.25);

    let player_render = SpriteRender {
        sprite_sheet: handles.player_sprites_handle,
        sprite_number: 0,
    };

    let boss_render = SpriteRender {
        sprite_sheet: handles.enemy_sprites_handle.clone(),
        sprite_number: 0,
    };

    let square_render = SpriteRender {
        sprite_sheet: handles.enemy_sprites_handle.clone(),
        sprite_number: 1,
    };

    let flying_render = SpriteRender {
        sprite_sheet: handles.enemy_sprites_handle,
        sprite_number: 2,
    };

    // TODO: maybe it's better not to have the x or y coordinates here.
    // using the relative position as a percentage would let us multiply
    // it by the computed playable area dimensions instead
    for rec in level_metadata.get_layout() {
        let (entity_type, x_percentage, y_percentage) = rec;
        let cleanup_tag = CleanupTag {};
        // these use logical width/height, which comes from the screen
        // dimensions resource. it is computed by that resource and does
        // not match the intended resolution in display_config.ron
        // on hidpi monitors
        let (x_pos, y_pos) = playable_area.relative_coordinates(x_percentage, y_percentage);
        //info!("x, y are {}, {}", &x_pos, &y_pos);
        let position = Translation3::new(x_pos, y_pos, 0.0);
        let transform = Transform::new(position, rotation, scale);

        match entity_type {
            EntityType::Boss => {
                world
                    .create_entity()
                    .with(handles.boss_prefab_handle.clone())
                    .with(boss_render.clone())
                    .with(transform)
                    .with(cleanup_tag)
                    .build();
            },
            EntityType::SquareEnemy => {
                world
                    .create_entity()
                    .with(handles.enemy_prefab_handle.clone())
                    .with(square_render.clone())
                    .with(transform)
                    .with(cleanup_tag)
                    .build();
            },
            EntityType::FlyingEnemy => {
                world
                    .create_entity()
                    .with(handles.flying_enemy_prefab_handle.clone())
                    .with(flying_render.clone())
                    .with(transform)
                    .with(cleanup_tag)
                    .build();
            },
            EntityType::Player => {
                world
                    .create_entity()
                    .with(handles.player_prefab_handle.clone())
                    .with(player_render.clone())
                    .with(transform)
                    .with(cleanup_tag)
                    .build();
            },
        }
    }
}
