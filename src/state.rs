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

use crate::level;

use log::info;

#[derive(new)]
pub struct GameplayState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,

    // we may need a more generic way to track victory conditions
    // at some point, but enemy count works for now
    pub enemy_count: i32,

    // keeps track of all the levels in our game
    pub levels: level::Levels,

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
    pub fn increment_by(&mut self, amt: i32) {
        self.count += amt;
    }

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

        // get a handle on the sprite sheet
        let enemy_sprite_sheet_handle = load_sprite_sheet(world, "enemy_sprites");

        self.enemy_sprites_handle = Some(enemy_sprite_sheet_handle);

        // need to register this type of entry before init
        world.register::<Player>();
        world.register::<Laser>();
        world.register::<Enemy>();
        world.register::<Collider>();
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

        let enemy_count = EnemyCount { count: 0 };

        world.insert(enemy_count);
    }

    // need to review https://docs.amethyst.rs/stable/amethyst/prelude/struct.World.html
    // for other options
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // probably a better way to do this, but we get the count as read only first
        // to avoid borrowing data.world
        let enemy_count = (*data.world.fetch::<EnemyCount>()).clone();
        //info!("enemy count is: {:?}", enemy_count);

        if enemy_count.count == 0 {
            // short form random thoughts:
            // the most basic implementation would be something like --
            // world.delete_all()
            // init_next_level()
            // this means I need to set the new enemy count by querying the level
            // information. remember to delete flying enemies if they go offscreen.
            // then have a system update a resource to decrement the enemy count
            // add a loop here to delete game entities
            // and grab the next level in some set of levels
            // make sure to add a few warmup levels

            // those flying enemies though... they need to be deleted off screen

            // longer form random thoughts:
            // TODO: choose the next level from some vector of levels
            // clear the game area? change the backdrop, remove the player, etc
            // initialize the next level, but possibly not the player yet
            // some kind of countdown so the player can prepare
            // does this mean we have a level complete transition screen?
            // delete everything, new game state, start over

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

                    let mut write_enemy_count = data.world.write_resource::<EnemyCount>();
                    write_enemy_count.increment_by(new_count);
                },
                None => {}, // info!("game over!!!"),
            }
        }

        Trans::None
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
fn load_sprite_sheet(world: &mut World, name: &str) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            format!("sprites/{}.png", name),
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("sprites/{}.ron", name),
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

// this could return the number of enemies generated, and a system
// could reduce that number as they're defeated
fn init_level(
    world: &mut World,
    entity_recs: Vec<level::EntityRecord>,
    prefab_handle: Handle<Prefab<EnemyPrefab>>,
    flying_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    sprite_sheet_handle: Handle<SpriteSheet>,
    player_prefab_handle: Handle<Prefab<PlayerPrefab>>,
) -> i32 {
    // well, this feels quite destructive
    //world.delete_all();

    //let entities = world.entities();
    //info!("entities: {:?}", entities);
    // clear existing players in storage
    // this kind of works, but not really
    // we should clear game state between levels in a smarter way
    world.write_storage::<Player>().clear();

    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(5.0, 5.0, 5.0);

    let blob_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 1,
    };

    let flying_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };

    let mut count = 0;

    for rec in entity_recs {
        if let (level::EntityType::BlobEnemy, x, y) = rec {
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

        if let (level::EntityType::FlyingEnemy, x, y) = rec {
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

        if let (level::EntityType::Player, x, y) = rec {
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
