/// Empty structs are a convenient way to "tag" an entity
/// so that it can be found in a system later. In terms of implementation
/// though these are plain old components, except using `NullStorage`
/// because we aren't storing a real value.
use amethyst::ecs::prelude::{Component, NullStorage};

/// Tags our background entity, which otherwise is one of many
/// entities with a sprite render and transform)
#[derive(Debug, Default)]
pub struct BackgroundTag;

impl Component for BackgroundTag {
    type Storage = NullStorage<Self>;
}

/// This lets us tag entities that should be cleaned up in the `on_stop`
/// method of our main `gameplay.rs` state (cleanup idea borrowed from
/// https://github.com/allora/breakout -- check it out!)
#[derive(Debug, Default)]
pub struct CleanupTag;

impl Component for CleanupTag {
    type Storage = NullStorage<Self>;
}

/// Used for the main game camera to make it easier to find.
#[derive(Debug, Default)]
pub struct CameraTag;

impl Component for CameraTag {
    type Storage = NullStorage<Self>;
}
