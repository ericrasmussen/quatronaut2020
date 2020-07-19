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

use log::info;

#[derive(new)]
pub struct GameplayState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,

    // handle to clone for the sprite sheet containing enemies
    #[new(default)]
    pub enemy_sprites_handle: Option<Handle<SpriteSheet>>,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub enemy_prefab_handle: Option<Handle<Prefab<EnemyPrefab>>>,

    #[new(default)]
    pub flying_enemy_prefab_handle: Option<Handle<Prefab<EnemyPrefab>>>,
    /// Haven't decided how/when to spawn enemy waves yet. This
    /// lets us spawn after a certain amount of time has elapsed,
    /// but will probably be replaced with something that spawns based
    /// on score, or launches a new wave when the enemy count is 0.
    pub wave_timer: f32,
    // player prefab. we could also use a config and one-time instantiation,
    // although at least for testing it's nice to spawn players as needed
    #[new(default)]
    pub player_prefab_handle: Option<Handle<Prefab<PlayerPrefab>>>,
}

// this is all kind of hacky... if this is going to be a global resource
// (which does kind of make sense, given how much game logic depends on the
// player's current position), then it should be used to update the player
// transform. right now the player system has to update this and the transform.
#[derive(Clone)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for PlayerPosition {
    fn default() -> Self {
        PlayerPosition { x: 1024.0, y: 768.0 }
    }
}

impl Component for PlayerPosition {
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
        world.register::<PlayerPosition>();

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

        // Create one player
        world.create_entity().with(player_prefab_handle.clone()).build();

        // this is a really hacky way to use coordinates as a resource. if this
        // approach works out then the spawn enemy logic should probably
        // move to a system, or at the very least the player transform becomes
        // a resource.
        let position = PlayerPosition { x: 1024.0, y: 768.0 };
        world.insert(position);

        self.player_prefab_handle = Some(player_prefab_handle);
    }

    // need to review https://docs.amethyst.rs/stable/amethyst/prelude/struct.World.html
    // for other options
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // check the time in a separate scope we can use `data.world` later
        {
            let time = data.world.fetch::<Time>();
            self.wave_timer -= time.delta_seconds();
        }
        let position = (*data.world.fetch::<PlayerPosition>()).clone();

        if self.wave_timer <= 0.0 {
            // set the timer to 10 seconds before the next wave starts.
            // again, mostly a placeholder until deciding on what makes sense for
            // actual gameplay
            self.wave_timer = 10.0;
            // TODO: decide how to handle unwrapping here, or if we even
            // need an `Option` type (since we shouldn't be this far into playing
            // the game if we didn't get this required prefab)
            init_enemy_wave(
                position,
                data.world,
                self.enemy_prefab_handle.clone().unwrap(),
                self.flying_enemy_prefab_handle.clone().unwrap(),
                self.enemy_sprites_handle.clone().unwrap(),
            );
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

fn init_enemy_wave(
    position: PlayerPosition,
    world: &mut World,
    prefab_handle: Handle<Prefab<EnemyPrefab>>,
    flying_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    sprite_sheet_handle: Handle<SpriteSheet>,
) {
    // TODO: ok, we have coordinates! wave logic can spawn around it now.
    info!("{:?}", position.x);

    // Create one set of entities from the prefab.
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(5.0, 5.0, 5.0);

    let blob_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 1,
    };

    let flying_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0,
    };

    // can't decide on a direction for spawning (designing levels, randomizing, etc).
    // these locations work for now
    // had around 45 total, modifying by 250 on x or y
    let mut blob_locations: Vec<(f32, f32)> = vec![
        // spawn points above the player
        (55.0, 1400.0), (1024.0, 1300.0), (1800.0, 1350.0),
        // spawn points below the player
        (250.0, 80.0), (725.0, 50.0), (1000.0, 10.0), (1500.0, 500.0),
        // spawn points on the right
        (1500.0, 700.0), (1650.0, 350.0), (1700.0, 1200.0), (1400.0, 1400.0),
        // spawn points on the left
        (30.0, 720.0), (400.0, 425.0), (200.0, 1100.0), (100.0, 1300.0),
    ];

    // here is where we need to double check no one collides with the player and then
    // shift them over a bit, and/or we could just use this opportunity to reset the player
    // to the center (note: will need the player transform to be based on the shared position
    // resource for this to work)
    for (x, y) in blob_locations.drain(..) {
        let position = Translation3::new(x, y, 0.0);
        let transform = Transform::new(position, rotation, scale);
        world
            .create_entity()
            .with(prefab_handle.clone())
            .with(blob_render.clone())
            .with(transform)
            .build();
    }
    // off-screen flying units
    let mut offset = 0.0;
    (0 .. 10).for_each(|_| {
        let position = Translation3::new(-90.0, offset, 0.0);
        offset += 250.0;
        let transform = Transform::new(position, rotation, scale);
        world
            .create_entity()
            .with(flying_prefab_handle.clone())
            .with(flying_render.clone())
            .with(transform)
            .build();
    });
}
