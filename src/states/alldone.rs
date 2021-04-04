//! This game state is for The End, Game Over, You Won,
//! All is Well, or however else you encapsulate the competing notions of
//! total domination over your foes vs utter defeat at the (metaphorical) hands
//! of slow-moving polygons.
//! tl;dr this is where you go for "You Win" or "Game Over" screens
use amethyst::{
    ecs::Entity,
    input::is_close_requested,
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
};

use crate::{resources::gameconfig::GameConfig, states::menu::MainMenu};

use derive_new::new;

const BUTTON_MENU: &str = "menu";

/// This struct tracks the current status of the game and the relevant
/// UI elements for the game over and you win screens.
#[derive(Debug, new)]
pub struct AllDone {
    // this is still how we're passing around level info to the menu,
    // and one day when I have infinite time I'll make this less clunky
    pub game_config: GameConfig,

    // whether we've won or not
    pub achieved_victory: bool,

    #[new(default)]
    button_menu: Option<Entity>,

    #[new(default)]
    ui_root: Option<Entity>,
}

impl SimpleState for AllDone {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        let menu_path = if self.achieved_victory {
            "ui/you_win.ron"
        } else {
            "ui/game_over.ron"
        };
        self.ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create(menu_path, ())));
    }

    fn update(&mut self, state_data: &mut StateData<'_, GameData>) -> SimpleTrans {
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = state_data;

        if self.button_menu.is_none() {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_menu = ui_finder.find(BUTTON_MENU);
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
                } else {
                    Trans::None
                }
            },
            StateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.button_menu {
                    log::info!("[Trans::Replace] Back to the menu");
                    Trans::Replace(Box::new(MainMenu::new(self.game_config.clone(), false)))
                } else {
                    Trans::None
                }
            },
            _ => Trans::None,
        }
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        // after destroying the current UI, invalidate references as well
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove MainMenu");
        }
    }
}
