/// This module implements and initializes game states to be used
/// by main.rs
use amethyst::{
    assets::{AssetStorage, Handle, Loader, Prefab, PrefabLoader, ProgressCounter, RonFormat},
    core::math::{Translation3, UnitQuaternion, Vector3},
    core::{transform::Transform, Time},
    ecs::{storage::DenseVecStorage, Component},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

use derive_new::new;

use crate::entities::{
    enemy::{Enemy, EnemyPrefab},
    laser::Laser,
    player::{Player, PlayerPrefab},
};

use crate::components::collider::Collider;

use crate::level::{EntityRecord, EntityType, LevelComplete, Levels};

use log::info;

#[derive(new)]
pub struct GameplayState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,

    // keeps track of all the levels in our game
    pub levels: Levels,

    // not clear yet if we need to treat the single background image as a sprite
    // sheet
    #[new(default)]
    pub background_sprite_handle: Option<Handle<SpriteSheet>>,

    // handle to clone for the sprite sheet containing enemies
    #[new(default)]
    pub enemy_sprites_handle: Option<Handle<SpriteSheet>>,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub enemy_prefab_handle: Option<Handle<Prefab<EnemyPrefab>>>,

    #[new(default)]
    pub flying_enemy_prefab_handle: Option<Handle<Prefab<EnemyPrefab>>>,
    // player prefab. we could also use a config and one-time instantiation,
    // although at least for testing it's nice to spawn players as needed
    #[new(default)]
    pub player_prefab_handle: Option<Handle<Prefab<PlayerPrefab>>>,
}

#[derive(Clone, Debug)]
pub struct EnemyCount {
    pub count: i32,
}

impl EnemyCount {
    pub fn decrement_by(&mut self, amt: i32) {
        self.count -= amt;
    }
}

impl Default for EnemyCount {
    fn default() -> Self {
        EnemyCount { count: 0 }
    }
}

impl Component for EnemyCount {
    type Storage = DenseVecStorage<Self>;
}

pub struct PausedState;

impl SimpleState for GameplayState {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Place the camera
        init_camera(world, &dimensions);

        // render the background
        // TODO: make this.. not awful? not clear that we actually need to save
        // the handle in the game state, so this may be overly cautious (based on
        // errors where the game engine would lose a reference to a sprite sheet handle)
        let bg_sprite_sheet_handle = load_sprite_sheet(world, "background", &mut self.progress_counter);
        self.background_sprite_handle = Some(bg_sprite_sheet_handle);
        init_background(world, &dimensions, self.background_sprite_handle.clone().unwrap());

        // get a handle on the sprite sheet
        let enemy_sprite_sheet_handle = load_sprite_sheet(world, "enemy_sprites", &mut self.progress_counter);
        self.enemy_sprites_handle = Some(enemy_sprite_sheet_handle);

        // need to register this type of entry before init
        world.register::<Player>();
        world.register::<Laser>();
        world.register::<Enemy>();
        world.register::<Collider>();
        world.register::<LevelComplete>();
        world.register::<EnemyCount>();

        let enemy_prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load("prefabs/enemy.ron", RonFormat, &mut self.progress_counter)
        });

        let flying_enemy_prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load("prefabs/flying_enemy.ron", RonFormat, &mut self.progress_counter)
        });

        // keep a handle on the enemies so they don't get out of control
        self.enemy_prefab_handle = Some(enemy_prefab_handle);

        // keep a handle on the enemies so they don't get out of control
        self.flying_enemy_prefab_handle = Some(flying_enemy_prefab_handle);

        // player prefab instantiation
        let player_prefab_handle = world.exec(|loader: PrefabLoader<'_, PlayerPrefab>| {
            loader.load("prefabs/player.ron", RonFormat, &mut self.progress_counter)
        });

        self.player_prefab_handle = Some(player_prefab_handle);

        // TODO: now that we don't change state, this should be 0 or default again
        let enemy_count = EnemyCount { count: 6 };
        world.insert(enemy_count);

        let level_complete = LevelComplete::default();
        info!("inserting new level complete struct: {:?}", level_complete);
        world.insert(level_complete);
    }

    // need to review https://docs.amethyst.rs/stable/amethyst/prelude/struct.World.html
    // for other options
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // we set the time scale to 0 until all assets are loaded. this is known as
        // the "I wasn't sure how to add a loading screen" technique
        if self.progress_counter.is_complete() {
            data.world.write_resource::<Time>().set_time_scale(1.0);
        } else {
            data.world.write_resource::<Time>().set_time_scale(0.0);
        }

        // probably a better way to do this, but we get the count as read only first
        // to avoid borrowing data.world
        let enemy_count = (*data.world.fetch::<EnemyCount>()).clone();
        //info!("enemy count is: {:?}", enemy_count);

        // this is our victory condition that lets us know the player finished
        // the level
        if enemy_count.count == 0 {
            info!("enemy count reached 0");
            data.world.write_resource::<LevelComplete>().success = true;
        }

        let level_complete = (*data.world.fetch::<LevelComplete>()).clone();

        // however, we're not ready for the next level until multiple conditions
        // are met, so here we defer to `level_complete` (systems will write to
        // this too)
        if level_complete.ready_for_next_level() {
            let next_level = self.levels.pop();

            match next_level {
                Some(level_entities) => {
                    let new_count = init_level(
                        data.world,
                        level_entities,
                        self.enemy_prefab_handle.clone().unwrap(),
                        self.flying_enemy_prefab_handle.clone().unwrap(),
                        self.enemy_sprites_handle.clone().unwrap(),
                        self.player_prefab_handle.clone().unwrap(),
                    );

                    {
                        let mut write_enemy_count = data.world.write_resource::<EnemyCount>();
                        write_enemy_count.count = new_count;
                        //info!("new enemy count is: {}", new_count);
                    }

                    {
                        let mut write_level_status = data.world.write_resource::<LevelComplete>();
                        write_level_status.start_over();
                        info!("current level complete resource says: {:?}", *write_level_status);
                    }
                },
                None => {}, // info!("game over!!!"),
            }

            Trans::None
        }
        // otherwise, nothing to see here folks!
        else {
            Trans::None
        }
    }

    fn handle_event(&mut self, data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            // simple pausing system that works only as long as we use delta timing
            //if let Some((VirtualKeyCode::P, _)) = get_key(&event) {
            if is_key_down(&event, VirtualKeyCode::P) {
                data.world.write_resource::<Time>().set_time_scale(0.0);
                return Trans::Push(Box::new(PausedState));
            }
        }

        // Keep going
        Trans::None
    }
}

// the state for pausing the game and going back to it

impl SimpleState for PausedState {
    fn handle_event(&mut self, data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::P) {
                // Go back to the `GameplayState` and reset the time scale
                data.world.write_resource::<Time>().set_time_scale(1.0);
                return Trans::Pop;
            }
        }

        // Escape isn't pressed, so we stay in this `State`.
        Trans::None
    }
}

// enemy_sprites
fn load_sprite_sheet(world: &mut World, name: &str, progress_counter: &mut ProgressCounter) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            format!("sprites/{}.png", name),
            ImageFormat::default(),
            progress_counter,
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("sprites/{}.ron", name),
        SpriteSheetFormat(texture_handle),
        // should be progress_counter
        (),
        &sprite_sheet_store,
    )
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    // many amethyst examples show using dimensions here, but it turns out we want the
    // intended dimensions (say, based on sprite sizes) and not the computed dimensions
    // (which are affected by hidpi and other factors, and may not be what we intended)
    world
        .create_entity()
        .with(Camera::standard_2d(1920.0, 1080.0))
        .with(transform)
        .build();
}

// render the background
fn init_background(world: &mut World, dimensions: &ScreenDimensions, bg_sprite_sheet_handle: Handle<SpriteSheet>) {
    // the z value is set to -25.0 for the position, to make sure the background stays in the back
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    // TODO: figure out if there's a better way to handle the image. when using a retina display
    // it changes the dimensions of the window size, which means the background image won't fit.
    // ideally there's some way to reliably compute the difference (say, actual
    // window height / image height as the scaling factor for height)
    info!("all dimension info: {:?}", dimensions);
    //    let scale = Vector3::new(1.0 * 1.5, 1.0 * dimensions.aspect_ratio(), 1.0);
    let scale = Vector3::new(1.0, 1.0, 1.0);
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, -25.0);
    let transform = Transform::new(position, rotation, scale);

    let bg_render = SpriteRender {
        sprite_sheet: bg_sprite_sheet_handle.clone(),
        sprite_number: 0,
    };

    world.create_entity().with(bg_render).with(transform).build();
}

// this could return the number of enemies generated, and a system
// could reduce that number as they're defeated
fn init_level(
    world: &mut World,
    entity_recs: Vec<EntityRecord>,
    prefab_handle: Handle<Prefab<EnemyPrefab>>,
    flying_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_prefab_handle: Handle<Prefab<PlayerPrefab>>,
) -> i32 {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(0.15, 0.15, 0.15);

    let blob_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 1,
    };

    let flying_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 2,
    };

    let mut count = 0;

    for rec in entity_recs {
        if let (EntityType::BlobEnemy, x, y) = rec {
            let position = Translation3::new(x, y, 0.0);
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(prefab_handle.clone())
                .with(blob_render.clone())
                .with(transform)
                .build();

            count += 1;
        }

        if let (EntityType::FlyingEnemy, x, y) = rec {
            let position = Translation3::new(x, y, 0.0);
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(flying_prefab_handle.clone())
                .with(flying_render.clone())
                .with(transform)
                .build();

            count += 1;
        }

        if let (EntityType::Player, x, y) = rec {
            let position = Translation3::new(x, y, 0.0);
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(player_prefab_handle.clone())
                .with(transform)
                .build();
        }
    }

    // should probably return this in a more helpful struct,
    // but as long as we make level decisions based on enemy
    // counts it's fine
    count
}
