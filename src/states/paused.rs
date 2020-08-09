/// The `states::gameplay::GameplayState` module uses a dispatcher to ensure
/// its systems only run when it is the active state. Consequently, this
/// paused state doesn't need to do anything to stop all the game action.
/// Eventually though it'd be good to have an overplay or "press p to continue"
/// message.
use amethyst::{
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
};

pub struct PausedState;

// wait for someone to press the P key so we can get back to the game
impl SimpleState for PausedState {
    fn handle_event(&mut self, _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::P) {
                return Trans::Pop;
            }
        }
        // if the user hasn't pressed P, keep this state active
        Trans::None
    }
}
