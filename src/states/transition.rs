/// This state can be pushed on top of `GameplayState`
/// and popped as needed. For now its main purpose is having
/// a kind of cutscene/level complete transition so that
/// progressing to the next level isn't so jarring.
use amethyst::{
    assets::Handle,
    core::math::{Translation3, UnitQuaternion, Vector3},
    core::{transform::Transform, ArcThreadPool},
    ecs::prelude::{Dispatcher, DispatcherBuilder, Join},
    ecs::world::EntitiesRes,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet, Transparent},
    window::ScreenDimensions,
};

use derive_new::new;

use crate::{
    components::{
        fade::{Fade, FadeStatus, Fader},
        perspective::Perspective,
        tags::BackgroundTag,
    },
    resources::level::Levels,
    states::{
        gameplay::{GameplayMode, GameplayState},
        paused::PausedState,
    },
    systems::{CameraShakeSystem, FadeSystem},
};

use log::info;

/// This state will be pushed on top of `GameplayState` to give more
/// control over level transitions, and, based on the meta level
/// resource, display some kind of cutscene (really, just moving the
/// player to an exit marker on completion)
#[derive(new)]
pub struct TransitionState<'a, 'b> {
    #[new(default)]
    pub dispatcher: Option<Dispatcher<'a, 'b>>,
    pub overlay_sprite_handle: Handle<SpriteSheet>,
    pub levels: Levels,
    pub perspective_shift: Option<Perspective>,
}

impl<'a, 'b> SimpleState for TransitionState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // creates a dispatcher to collect systems specific to this state
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(FadeSystem, "fade_system", &[]);
        dispatcher_builder.add(CameraShakeSystem, "camera_shake_system", &[]);

        // builds and sets up the dispatcher
        let mut dispatcher = dispatcher_builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);

        world.register::<Perspective>();
        if let Some(perspective) = &self.perspective_shift {
            info!("adding some perspective");
            world.insert(*perspective);
        }

        // this is all a little over complicated, but the status is a shared
        // resource to track if fading has completed
        world.register::<FadeStatus>();
        world.insert(FadeStatus::default());

        // insert a new fader to start darkening the screen
        world.register::<Fader>();
        let default_fader = Fader::new(0.001, Fade::Darken);
        world.entry::<Fader>().or_insert_with(|| default_fader);

        // initialize the overlay image
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();
        init_overlay(
            world,
            &dimensions,
            self.overlay_sprite_handle.clone(),
            self.perspective_shift,
        );
    }

    // TODO: maybe check for the Perspective and then go back to gameplay (damaged art
    // mode)
    // should the actual background change here though?
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        if let Some(_p) = &self.perspective_shift {
            let perspective = data.world.read_resource::<Perspective>();
            if perspective.is_reversing() {
                let mut sprites = data.world.write_storage::<SpriteRender>();
                let backgrounds = data.world.read_storage::<BackgroundTag>();
                for (sprite, _bg) in (&mut sprites, &backgrounds).join() {
                    sprite.sprite_number = 1;
                }
            }
            // special case to return early if we're done with our scaling and shaking
            if perspective.is_completed() {
                return Trans::Switch(Box::new(GameplayState::new(
                    self.levels.clone(),
                    GameplayMode::EndlessMode,
                )));
            }
        }

        let mut fade_status = data.world.write_resource::<FadeStatus>();

        if fade_status.is_completed() {
            fade_status.clear();

            Trans::Switch(Box::new(GameplayState::new(
                self.levels.clone(),
                GameplayMode::LevelMode,
            )))
        } else {
            Trans::None
        }
    }

    // TODO: remove the Perspective resource here too
    fn on_stop(&mut self, data: StateData<GameData>) {
        // state items that should be cleaned up (players, entities, lasers,
        // projectiles) should all be marked with `CleanupTag` and removed
        // here when this state ends
        let entities = data.world.read_resource::<EntitiesRes>();
        let faders = data.world.read_storage::<Fader>();

        for (entity, _tag) in (&entities, &faders).join() {
            let err = format!("unable to delete entity: {:?}", entity);
            entities.delete(entity).expect(&err);
        }
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
fn init_overlay(
    world: &mut World,
    dimensions: &ScreenDimensions,
    overlay_sprite_handle: Handle<SpriteSheet>,
    perspective_shift: Option<Perspective>,
) {
    let rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    let scale = Vector3::new(100.0, 100.0, 1.0);
    let position = Translation3::new(dimensions.width() * 0.5, dimensions.height() * 0.5, 0.0);
    let transform = Transform::new(position, rotation, scale);
    let fader = Fader::new(6.0, Fade::Darken);
    let tint = Tint(Srgba::new(0.0, 0.0, 0.0, 0.0));
    let overlay_render = SpriteRender {
        sprite_sheet: overlay_sprite_handle,
        sprite_number: 0,
    };

    match perspective_shift {
        None => {
            world
                .create_entity()
                .with(overlay_render)
                .with(transform)
                .with(Transparent)
                .with(tint)
                .with(fader)
                .build();
        },
        Some(perspective) => {
            world
                .create_entity()
                .with(overlay_render)
                .with(transform)
                .with(Transparent)
                .with(tint)
                .with(perspective)
                .build();
        },
    }
}
