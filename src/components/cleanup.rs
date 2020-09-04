/// This struct is used as a tag to mark which items
/// should be cleaned up `on_stop` in `gameplay.rs`.
/// Idea based on https://github.com/allora/breakout
use amethyst::ecs::prelude::{Component, DenseVecStorage};

#[derive(Debug)]
pub struct CleanupTag;

impl Component for CleanupTag {
    type Storage = DenseVecStorage<Self>;
}
