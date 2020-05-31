/// This module implements and initializes game states to be used
/// by main.rs
use amethyst::{
    assets::{AssetStorage, Loader, Prefab, PrefabLoader, Handle, ProgressCounter, RonFormat},
    core::math::{Translation3, UnitQuaternion, Vector3},
    core::transform::Transform,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};
use amethyst::core::timing::Time;

use derive_new::new;
//use amethyst::input::get_key;
//use log::info;

use crate::entities::{enemy::{Enemy, EnemyPrefab}, laser::Laser, player::Player};

#[derive(new)]
pub struct GameplayState {
    /// Tracks loaded assets.
    #[new(default)]
    pub progress_counter: ProgressCounter,
    /// Handle to the loaded prefab.
    #[new(default)]
    pub prefab_handle: Option<Handle<Prefab<EnemyPrefab>>>,
    /// Haven't decided how/when to spawn enemy waves yet. This
    /// lets us spawn after a certain amount of time has elapsed,
    /// but will probably be replaced with something that spawns based
    /// on score, or launches a new wave when the enemy count is 0.
    pub wave_timer: f32,
}

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

        // TODO: why is this enemy?
        let prefab_handle = world.exec(|loader: PrefabLoader<'_, EnemyPrefab>| {
            loader.load(
                "prefabs/enemy.ron",
                RonFormat,
                &mut self.progress_counter,
            )
        });

        // Create one set of entities from the prefab.
        (0..1).for_each(|_| {
            world
                .create_entity()
                .with(prefab_handle.clone())
                .build();
        });

        self.prefab_handle = Some(prefab_handle);

        // get a handle on the sprite sheet
        let sprite_sheet_handle = load_sprite_sheet(world, "sprite_sheet");

        // get the enemy sprite sheet. it's separate for now since the assets
        // will all be changing anyway
        //let enemy_sprite_sheet_handle = load_sprite_sheet(world, "enemy_sprites");
        // need to register this type of entry before init
        world.register::<Player>();
        world.register::<Laser>();
        world.register::<Enemy>();
        init_characters(world, sprite_sheet_handle, &dimensions);
        //init_enemies(world, enemy_sprite_sheet_handle);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // check the time in a separate scope we can use `data.world` later
        {
            let time = data.world.fetch::<Time>();
            self.wave_timer -= time.delta_seconds();
        }
        if self.wave_timer <= 0.0 {
            // set the timer to 10 seconds before the next wave starts.
            // again, mostly a placeholder until deciding on what makes sense for
            // actual gameplay
            self.wave_timer = 20.0;
            // TODO: decide how to handle unwrapping here, or if we even
            // need an `Option` type (since we shouldn't be this far into playing
            // the game if we didn't get this required prefab)
            init_enemy_wave(data.world, self.prefab_handle.clone().unwrap());

        }
        Trans::None
    }

    fn handle_event(&mut self, mut _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            // Listen to any key events
            //if let Some(event) = get_key(&event) {
            //    info!("handling key event: {:?}", event);
            //}

            // TODO: eventually need to add a menu type screen for state
            // transitions. this doesn't really work because it doesn't clean up
            // the current state
            //if let Some((VirtualKeyCode::R, _)) = get_key(&event) {
            //    return Trans::Switch(Box::new(MyState));
            //}
        }

        // Keep going
        Trans::None
    }
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

fn init_enemy_wave(world: &mut World, prefab_handle: Handle<Prefab<EnemyPrefab>>) {
        // Create one set of entities from the prefab.
        let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
        let scale = Vector3::new(5.0, 5.0, 5.0);
        let mut offset = 250.0;

        // bottom wave
        (0..15).for_each(|_| {
            let position = Translation3::new(offset, 20.0, 0.0);
            offset += 250.0;
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(prefab_handle.clone())
                .with(transform)
                .build();
        });
        // top wave
        offset = 0.0;
        (0..15).for_each(|_| {
            let position = Translation3::new(offset, 1600.0, 0.0);
            offset += 250.0;
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(prefab_handle.clone())
                .with(transform)
                .build();
        });
        // left wave
        offset = 0.0;
        (0..10).for_each(|_| {
            let position = Translation3::new(0.0, offset, 0.0);
            offset += 250.0;
            let transform = Transform::new(position, rotation, scale);
            world
                .create_entity()
                .with(prefab_handle.clone())
                .with(transform)
                .build();
        });

}

// sprite_sheet
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

// for now, all this does is create a player entity.
// ideally we'll move even the player entity data (speed, fire_rate, sprite sheet)
// to a prefab.
fn init_characters(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>, dimensions: &ScreenDimensions) {
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, 0.0);
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let scale = Vector3::new(5.0, 5.0, 5.0);
    let player_transform = Transform::new(position, rotation, scale);

    // we can fire 4 times a second
    let fire_delay = 0.17;

    // Assign the sprites for the player
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0, // player is the first sprite in the sprite_sheet
    };

    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Player::new(10.0, fire_delay))
        .with(player_transform)
        .build();
}
