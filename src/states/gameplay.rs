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
    ui::{UiCreator, UiFinder, UiText},
    window::ScreenDimensions,
};

use derive_new::new;

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
        handles,
        handles::GameplayHandles,
        level::{EntityType, LevelMetadata, LevelStatus, Levels},
        music,
        playablearea::PlayableArea,
        playerstats::PlayerStats,
    },
    states::{mainmenu::MainMenu, paused::PausedState, transition::TransitionState},
    systems,
};

use rand::{thread_rng, Rng};

// could use separate states here, but they feel a little heavy. all the
// modes here share the same gameplay logic
#[derive(Clone, Debug, PartialEq)]
pub enum GameplayMode {
    LevelMode,
    TransitionMode,
    EndlessMode,
}

use GameplayMode::*;

/// Collects our state-specific dispatcher, progress counter for asset
/// loading, struct with gameplay handles, and levels. Note that the
/// levels are loaded via `main.rs` (since they can be created from a
/// config file without gameplay state knowledge)
#[derive(new)]
pub struct GameplayState<'a, 'b> {
    pub levels: Levels,

    pub sound_config: audio::SoundConfig,

    // default initializes this value with false
    #[new(default)]
    pub level_is_loaded: bool,

    pub gameplay_mode: GameplayMode,

    #[new(default)]
    pub ui_root: Option<Entity>,

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
        world.register::<PlayerStats>();

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
            &self.gameplay_mode,
            &dimensions,
            self.handles.clone().unwrap().background_sprite_handle,
        );

        // audio should not need to be initialized multiple times
        audio::initialize_audio(world, &self.sound_config);

        // setup our music player
        music::initialize_music(world);

        // we want to preserve palyer stats across levels, so only insert if it isn't there yet
        world.entry::<PlayerStats>().or_insert_with(PlayerStats::default);

        //
        let next_level_status = self.levels.pop();

        // setup the playable area. this is still messy but if we begin in small level mode,
        // we setup the constrained level mode
        let playable_area = match next_level_status {
            LevelStatus::SmallLevel(_) => PlayableArea::new(dimensions.width(), dimensions.height(), true),
            _ => PlayableArea::new(dimensions.width(), dimensions.height(), false),
        };

        world.insert(playable_area);

        let handles = self.handles.clone().expect("failure accessing GameplayHandles struct");

        // maybe use one of two configs here? and clean up each time
        // UI setup
        let ui_config = match next_level_status {
            LevelStatus::SmallLevel(_) => "ui/ui.ron",
            _ => "ui/ui_big.ron",
        };

        let ui_handle = world.exec(|mut creator: UiCreator<'_>| creator.create(ui_config, &mut self.progress_counter));
        self.ui_root = Some(ui_handle);

        //info!("gameplay mode is now: {:?}", &self.gameplay_mode);

        match &self.gameplay_mode {
            LevelMode => match next_level_status {
                LevelStatus::SmallLevel(next_level_metadata) => init_level(world, next_level_metadata, handles),
                LevelStatus::LargeLevel(next_level_metadata) => init_level(world, next_level_metadata, handles),
                LevelStatus::TransitionTime => self.gameplay_mode = TransitionMode,
                LevelStatus::AllDone => self.gameplay_mode = EndlessMode,
            },
            EndlessMode => {
                spawn_player_in_center(world, &dimensions, handles.clone());
                // TODO: spawn on timer via update()
                spawn_enemies(world, handles);
            },
            _ => {},
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

        // ui text handling
        if self.high_score_text.is_none() {
            data.world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("high_score") {
                    self.high_score_text = Some(entity);
                }
            });
        }
        // this must be used in a separate scope
        // but this should really be in a system anyway, I think
        let mut ui_text = data.world.write_storage::<UiText>();
        let score = data.world.read_resource::<PlayerStats>();

        {
            if let Some(high_score_text) = self.high_score_text.and_then(|entity| ui_text.get_mut(entity)) {
                high_score_text.text = format!("{}", score.get_score());
            }
        }

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
        // if self.gameplay_mode == EndlessMode {
        //     spawn_enemies(data.world, handles);
        // }

        // this branch decides whether or not to switch state. if a level is
        // loaded and all enemies are defeated, it's time to transition, otherwise
        // keep going
        if self.gameplay_mode == TransitionMode {
            Trans::Switch(Box::new(TransitionState::new(
                handles.overlay_sprite_handle,
                self.levels.clone(),
                self.sound_config.clone(),
                Some(Perspective::new(1.4, 0.3, audio::SoundType::LongTransition)),
            )))
        } else if total == 0 && self.level_is_loaded {
            Trans::Switch(Box::new(TransitionState::new(
                handles.overlay_sprite_handle,
                self.levels.clone(),
                self.sound_config.clone(),
                None,
            )))
        } else if self.gameplay_mode == EndlessMode {
            //info!("I could be updating now!!!!");
            // endless logic needs to go here
            Trans::None
        }
        // otherwise, nothing to see here folks!
        else {
            Trans::None
        }
    }

    // handles pausing (toggling the `p` key) and closing (window close or pressing escape)
    fn handle_event(&mut self, _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Push(Box::new(MainMenu::new(self.levels.clone(), self.sound_config.clone(), self.gameplay_mode.clone(), true)));
                //return Trans::Quit;
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
        // ui cleanup
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove UI elements");
        }
        self.ui_root = None;
    }
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);
    // many amethyst examples show using dimensions here, but it turns out we want the
    // intended dimensions (say, based on sprite sizes) and not the computed dimensions
    // (which are affected by hidpi and other factors, and may not be what we intended)
    // even though transforms can use the computed dimensions, the camera needs the requested
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
fn init_background(
    world: &mut World,
    gameplay_mode: &GameplayMode,
    dimensions: &ScreenDimensions,
    bg_sprite_sheet_handle: Handle<SpriteSheet>,
) {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    let scale = Vector3::new(1.0, 1.0, 1.0);
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, -25.0);
    let transform = Transform::new(position, rotation, scale);

    let sprite_number = if gameplay_mode == &EndlessMode { 1 } else { 0 };
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

// takes the current level metadata and gameplay handles, then adds
// all the associated entities and components to the world
fn spawn_player_in_center(world: &mut World, dimensions: &ScreenDimensions, handles: GameplayHandles) {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(0.25, 0.25, 0.25);

    let player_render = SpriteRender {
        sprite_sheet: handles.player_sprites_handle,
        sprite_number: 0,
    };

    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, 0.0);

    let transform = Transform::new(position, rotation, scale);
    let cleanup_tag = CleanupTag {};

    world
        .create_entity()
        .with(handles.player_prefab_handle)
        .with(player_render)
        .with(transform)
        .with(cleanup_tag)
        .build();
}

fn init_level(world: &mut World, level_metadata: LevelMetadata, handles: GameplayHandles) {
    // let (play_width, play_height) = {
    //     let playable_area = (*world.read_resource::<PlayableArea>()).clone();
    //     (playable_area.max_x, playable_area.max_y)
    // };
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
// takes the current level metadata and gameplay handles, then adds
// all the associated entities and components to the world
fn spawn_enemies(world: &mut World, handles: GameplayHandles) {
    let playable_area = (*world.read_resource::<PlayableArea>()).clone();
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(0.25, 0.25, 0.25);

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

    let (x_min, y_min) = playable_area.relative_coordinates(&0.12, &0.12);
    let (x_max, y_max) = playable_area.relative_coordinates(&0.88, &0.88);

    for n in 1 .. 12 {
        let mut rng = thread_rng();

        // for spawning we want x to be 0 or max and y to be anything
        // or, y to be 0 or max and x to be anything. this ends up
        // placing enemies around the edges, but if we were to spawn
        // continuously we'd still have to avoid spawning on a player
        let y_aligned = n % 2 == 0;

        let (x, y) = if y_aligned {
            let choose_min_x: bool = rng.gen();
            let x_value = if choose_min_x { x_min } else { x_max };
            let y_value = rng.gen_range(y_min, y_max);
            (x_value, y_value)
        } else {
            let choose_min_y: bool = rng.gen();
            let x_value = rng.gen_range(x_min, x_max);
            let y_value = if choose_min_y { y_min } else { y_max };
            (x_value, y_value)
        };

        //info!("computed x, y ({:?}, {:?})", x, y);
        let cleanup_tag = CleanupTag {};
        let position = Translation3::new(x, y, 0.0);
        let transform = Transform::new(position, rotation, scale);

        let entity_type: EntityType = rng.gen();

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
            _ => {},
        }
    }
}
