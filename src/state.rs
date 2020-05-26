/// This module implements and initializes game states to be used
/// by main.rs

use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::transform::Transform,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
    core::math::{Translation3, UnitQuaternion, Vector3},
};

// uncomment the lines below for logging key input
// use amethyst::input::get_key
// use log::info;

use crate::entities::player::Player;
use crate::entities::laser::Laser;

pub struct MyState;

impl SimpleState for MyState {
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
        let sprite_sheet_handle = load_sprite_sheet(world);

        // need to register this type of entry before init
        world.register::<Player>();
        world.register::<Laser>();
        init_characters(world, sprite_sheet_handle, &dimensions);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            // Listen to any key events
            //if let Some(event) = get_key(&event) {
            //    info!("handling key event: {:?}", event);
            //}

            // If you're looking for a more sophisticated event handling solution,
            // including key bindings and gamepad support, please have a look at
            // https://book.amethyst.rs/stable/pong-tutorial/pong-tutorial-03.html#capturing-user-input
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

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprites/sprite_sheet.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "sprites/sprite_sheet.ron", // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}


// for now, all this does is create a player entity.
// ideally we'll move even the player entity data (speed, fire_rate, sprite sheet)
// to a prefab.
fn init_characters(world: &mut World,
        sprite_sheet_handle: Handle<SpriteSheet>,
        dimensions: &ScreenDimensions) 
    {

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
