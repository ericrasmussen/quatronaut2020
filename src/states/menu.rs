//! The primary UI for starting new games, continuing games,
//! and quitting.
use amethyst::{
    ecs::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};

use derive_new::new;

use crate::{
    resources::{
        gameconfig::{GameConfig, GameplayMode},
        level::get_all_levels,
    },
    states::gameplay::GameplayState,
};

const BUTTON_START: &str = "start";
const BUTTON_CONTINUE: &str = "continue";
const BUTTON_QUIT: &str = "quit";

#[derive(Debug, new)]
pub struct MainMenu {
    pub game_config: GameConfig,
    // indicates whether or not there's a continue-able/active game
    pub active_game: bool,
    #[new(default)]
    ui_root: Option<Entity>,
    #[new(default)]
    button_start: Option<Entity>,
    #[new(default)]
    button_continue: Option<Entity>,
    #[new(default)]
    button_quit: Option<Entity>,
}

impl SimpleState for MainMenu {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        let menu_path = if self.active_game {
            "ui/menu.ron"
        } else {
            "ui/menu_no_continue.ron"
        };
        self.ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create(menu_path, ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_start.is_none() || self.button_continue.is_none() || self.button_quit.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_start = ui_finder.find(BUTTON_START);
                self.button_continue = ui_finder.find(BUTTON_CONTINUE);
                self.button_quit = ui_finder.find(BUTTON_QUIT);
            });
        }

        Trans::None
    }

    fn handle_event(&mut self, _: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else {
                    Trans::None
                }
            },
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.button_quit {
                    log::info!("[Trans::Switch] Quitting!");
                    return Trans::Quit;
                }
                if Some(target) == self.button_start {
                    log::info!("[Trans::Switch] Switching to New Game!");
                    // this here should be a clean copy of the levels for a new game
                    let mut new_game_config = self.game_config.clone();
                    let all_levels = get_all_levels(self.game_config.level_config.clone());
                    new_game_config.current_levels = all_levels;
                    new_game_config.gameplay_mode = GameplayMode::LevelMode;
                    // Switch doesn't work here for whatever reason, but Replace ensures we
                    // get a brand new `GameplayState`
                    return Trans::Replace(Box::new(GameplayState::new(new_game_config)));
                }
                if Some(target) == self.button_continue {
                    return Trans::Pop;
                }
                Trans::None
            },
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // after destroying the current UI, invalidate references as well (makes things cleaner)
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove MainMenu");
        }

        self.ui_root = None;
        self.button_start = None;
        self.button_continue = None;
        self.button_quit = None;
    }
}
