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

use rand::{thread_rng, Rng};

use crate::{
    components::{
        fade::{Fade, FadeStatus, Fader},
        cutscene::{Cutscene, CutsceneStatus},
        perspective::{Perspective, PerspectiveStatus},
        tags::{BackgroundTag, CleanupTag},
    },
    resources::{
        direction::Direction,
        gameconfig::{GameConfig, GameplayMode},
        playablearea::PlayableArea,
    },
    states::{gameplay::GameplayState, paused::PausedState},
    systems::{CameraShakeSystem, CameraZoomSystem, FadeSystem},
};

use log::info;

/// This state offers different ways to transition between levels.
/// If it's given a perspective shift, it'll rotate the camera on the z-axis
/// and play a sound. If it's given a cutscene, it'll zoom in, break some
/// glass, and zoom out to reveal a new background.
/// Otherwise it'll just do a quick fade to black and back.
/// NOTE: I dunno what'll happen if you give it a perspective shift and a
/// cutscene. Probably two sound effects at the same time, rotating and zooming
/// camera, and one of the two will cause an exit before the other is done.
/// So don't do that.
/// Or you know, if you're reading this, maybe just make a new enum or a
/// TransitionLike trait. I would, but I'm really busy writing comments right now.
#[derive(new)]
pub struct TransitionState<'a, 'b> {
    #[new(default)]
    pub dispatcher: Option<Dispatcher<'a, 'b>>,
    #[new(default)]
    pub glass_spawned: bool,

    pub overlay_sprite_handle: Handle<SpriteSheet>,
    pub glass_sprite_handle: Handle<SpriteSheet>,
    pub game_config: GameConfig,
    pub perspective_shift: Option<Perspective>,
    pub cutscene: Option<Cutscene>,
}

impl<'a, 'b> SimpleState for TransitionState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // creates a dispatcher to collect systems specific to this state
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(FadeSystem, "fade_system", &[]);
        dispatcher_builder.add(CameraShakeSystem, "camera_shake_system", &[]);
        dispatcher_builder.add(CameraZoomSystem, "camera_zoom_system", &[]);

        // builds and sets up the dispatcher
        let mut dispatcher = dispatcher_builder
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);

        world.register::<Perspective>();
        if let Some(perspective) = self.perspective_shift {
            world.insert(perspective);
        }

        world.register::<Cutscene>();
        if let Some(cutscene) = self.cutscene {
            world.insert(cutscene);
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
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        if let Some(_p) = &self.perspective_shift {
            let perspective = data.world.read_resource::<Perspective>();

            // return early if we're done with our scaling and shaking
            if perspective.status == PerspectiveStatus::Completed {
                let mut game_config = self.game_config.clone();
                game_config.gameplay_mode = GameplayMode::LevelMode;
                return Trans::Replace(Box::new(GameplayState::new(game_config)));
            }
        }

        if let Some(_c) = &self.cutscene {
            // separate scope here to avoid the immutable borrow and ensure
            // we're done with the world
            let cutscene = {
                let world_ref_cutscene = data.world.read_resource::<Cutscene>();
                *world_ref_cutscene
            };

            // change the background image if we've zoomed all the way in
            // and are getting ready to zoom out and reveal the larger background
            if cutscene.status == CutsceneStatus::Reversing {
                let mut sprites = data.world.write_storage::<SpriteRender>();
                let backgrounds = data.world.read_storage::<BackgroundTag>();
                for (sprite, _bg) in (&mut sprites, &backgrounds).join() {
                    sprite.sprite_number = 1;
                }
            } else if cutscene.status == CutsceneStatus::Completed {
                let mut game_config = self.game_config.clone();
                game_config.gameplay_mode = GameplayMode::LevelMode;
                return Trans::Replace(Box::new(GameplayState::new(game_config)));
            } else if cutscene.status == CutsceneStatus::Spawning && !self.glass_spawned {
                init_glass(data.world, self.glass_sprite_handle.clone());
                // make sure glass is only spawned once
                self.glass_spawned = true;
            }
        }

        let mut fade_status = data.world.write_resource::<FadeStatus>();

        // if we have any kind of non-fade transition, they determine when to switch
        // states
        let managed_scene = self.perspective_shift.is_some() || self.cutscene.is_some();

        if fade_status.is_completed() && !managed_scene {
            fade_status.clear();

            let mut game_config = self.game_config.clone();
            game_config.gameplay_mode = GameplayMode::LevelMode;

            Trans::Replace(Box::new(GameplayState::new(game_config)))
        } else {
            Trans::None
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // state items that should be cleaned up (players, entities, lasers,
        // projectiles) should all be marked with `CleanupTag` and removed
        // here when this state ends
        let entities = data.world.read_resource::<EntitiesRes>();
        let cleanup_tags = data.world.read_storage::<CleanupTag>();
        let faders = data.world.read_storage::<Fader>();

        for (entity, _tag) in (&entities, &cleanup_tags).join() {
            let err = format!("unable to delete entity: {:?}", entity);
            entities.delete(entity).expect(&err);
        }

        for (entity, _fader) in (&entities, &faders).join() {
            let err = format!("unable to delete entity: {:?}", entity);
            entities.delete(entity).expect(&err);
        }

        // make sure we clean up any perspective resources (that contain information
        // about shaking the camera or zooming in and out)
        if let Some(_perspective) = &self.perspective_shift {
            let perspectives = data.world.read_storage::<Perspective>();
            for (entity, _perspective) in (&entities, &perspectives).join() {
                let err = format!("unable to delete entity: {:?}", entity);
                entities.delete(entity).expect(&err);
            }
        }
        if let Some(_cutscene) = &self.cutscene {
            let cutscenes = data.world.read_storage::<Cutscene>();
            for (entity, _perspective) in (&entities, &cutscenes).join() {
                let err = format!("unable to delete entity: {:?}", entity);
                entities.delete(entity).expect(&err);
            }
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

fn init_glass(world: &mut World, glass_sprite_handle: Handle<SpriteSheet>) {

    let playable_area = (*world.read_resource::<PlayableArea>()).clone();

    let base_rotation = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);

    info!("inserting some glass shards now!");

    // the step by is mostly arbitrary based on what seems to look ok
    for x_coord in (-4..101).step_by(4) {
        for y_coord in (-4..101).step_by(4) {
            let cleanup_tag = CleanupTag {};

            let mut rng = thread_rng();
            let dir: Direction = rng.gen();

            // available glass sprites in glass_shards.{png,ron} are 0, 1, 2
            let sprite_num: usize = rng.gen_range(0, 2);

            let render = SpriteRender {
                sprite_sheet: glass_sprite_handle.clone(),
                sprite_number: sprite_num,
            };

            let x_pct: f32 = x_coord as f32 / 100.0;
            let y_pct: f32 = y_coord as f32 / 100.0;
            let (x_pos, y_pos) = playable_area.relative_coordinates(&x_pct, &y_pct);

            let position = Translation3::new(x_pos, y_pos, 0.0);

            let rotation = dir.direction_to_radians();
            // TODO: should the scale be randomized too?
            // let scale_factor = rng.gen_range(0.2, 0.8);
            // let scale = Vector3::new(scale_factor, scale_factor, scale_factor);
            let scale = Vector3::new(0.25, 0.25, 0.25);
            let mut transform = Transform::new(position, base_rotation, scale);

            // rotate based on the randomly chosen `Direction`
            transform.set_rotation_2d(rotation);
            world
                .create_entity()
                //.with(glass_component)
                .with(render)
                .with(transform)
                .with(cleanup_tag)
                .build();

        }
    }
}
