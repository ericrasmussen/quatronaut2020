/// This state can be pushed on top of `GameplayState`
/// and popped as needed. For now its main purpose is having
/// a kind of cutscene/level complete transition so that
/// progressing to the next level isn't so jarring.
use amethyst::{
    assets::Handle,
    core::math::{Translation3, UnitQuaternion, Vector3},
    core::{transform::Transform, ArcThreadPool},
    ecs::prelude::{Dispatcher, DispatcherBuilder},
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet, Transparent},
    window::ScreenDimensions,
};

use derive_new::new;

use crate::{
    resources::fade::{Fade, Fader},
    states::paused::PausedState,
    systems::FadeSystem,
};

//use log::info;

/// This state will be pushed on top of `GameplayState` to give more
/// control over level transitions, and, based on the meta level
/// resource, display some kind of cutscene (really, just moving the
/// player to an exit marker on completion)
#[derive(new)]
pub struct TransitionState<'a, 'b> {
    #[new(default)]
    pub dispatcher: Option<Dispatcher<'a, 'b>>,
    pub overlay_sprite_handle: Handle<SpriteSheet>,
}

impl<'a, 'b> SimpleState for TransitionState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // creates a dispatcher to collect systems specific to this state
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(FadeSystem, "fade_system", &[]);

        // builds and sets up the dispatcher
        let mut dispatcher = dispatcher_builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);

        // if there is no fader resource yet, assume we'll need to darken
        // the screen first
        world.register::<Fader>();
        let default_fader = Fader::new(0.001, Fade::Darken);
        world.entry::<Fader>().or_insert_with(|| default_fader);

        // initialize the overlay image
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();
        init_overlay(world, &dimensions, self.overlay_sprite_handle.clone());
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        // here's where we'll need to check the fade resource. if it's fully darkened or
        // fully faded, we can call .switch()
        // do we need a stopped state? or we could just delete instead of switch...
        // because we need to know when to pop the current state here too

        // but do we really want a transparent drawing over everything all the time?
        // that will have to be added on start and associated with a transform and fader
        // which the system will use

        // and this will be an if/else to pop state if needed
        Trans::None
    }

    // handles pausing (toggling the `p` key) and closing (window close or pressing escape)
    fn handle_event(&mut self, _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            if is_key_down(&event, VirtualKeyCode::P) {
                return Trans::Push(Box::new(PausedState));
            }
        }

        // no state changes required
        Trans::None
    }
}

// render the background, giving it a low z value so it renders under
// everything else
fn init_overlay(world: &mut World, dimensions: &ScreenDimensions, overlay_sprite_handle: Handle<SpriteSheet>) {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    let scale = Vector3::new(100.0, 100.0, 1.0);
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, 0.0);
    let transform = Transform::new(position, rotation, scale);
    let fader = Fader::new(0.25, Fade::Darken);
    let tint = Tint(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let overlay_render = SpriteRender {
        sprite_sheet: overlay_sprite_handle,
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(overlay_render)
        .with(transform)
        .with(Transparent)
        .with(tint)
        .with(fader)
        .build();
}
