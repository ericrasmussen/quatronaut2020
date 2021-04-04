//! There are a lot of handles to manage in a game. This struct
//! at least makes them easier to pass around. It does expose some
//! of the difficulties of mixing and matching prefabs with non-prefab
//! approaches. If I had time I would probably remove the prefabs
//! completely.
use amethyst::{
    assets::{AssetStorage, Handle, Loader, Prefab, ProgressCounter},
    prelude::*,
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
};

use crate::entities::{enemy::EnemyPrefab, player::PlayerPrefab};

/// The `GameplayState` needs to keep track of many prefab and spritesheet
/// handles to run. This struct mostly exists to organize all those handles
/// into one namespace.
#[derive(Clone, Debug)]
pub struct GameplayHandles {
    // gameplay bg image
    pub background_sprite_handle: Handle<SpriteSheet>,

    // image used for fade to black transitions
    pub overlay_sprite_handle: Handle<SpriteSheet>,

    // glass shard images used for ultra realistic background breaking effect
    pub glass_sprite_handle: Handle<SpriteSheet>,

    // handle to clone for the sprite sheet containing enemies
    pub enemy_sprites_handle: Handle<SpriteSheet>,

    // all the prefab handles
    pub enemy_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    pub flying_enemy_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    pub player_prefab_handle: Handle<Prefab<PlayerPrefab>>,
    pub player_hyper_prefab_handle: Handle<Prefab<PlayerPrefab>>,
    pub boss_prefab_handle: Handle<Prefab<EnemyPrefab>>,

    // handle to clone for the sprite sheet containing player and laser images
    pub player_sprites_handle: Handle<SpriteSheet>,
}

/// This relys on `gameplay.rs` to pass in the prefabs. It then loads all
/// the non-prefab spritesheets, and puts all the handles in one handy struct.
pub fn get_game_handles(
    world: &mut World,
    progress_counter: &mut ProgressCounter,
    enemy_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    flying_enemy_prefab_handle: Handle<Prefab<EnemyPrefab>>,
    player_prefab_handle: Handle<Prefab<PlayerPrefab>>,
    player_hyper_prefab_handle: Handle<Prefab<PlayerPrefab>>,
    boss_prefab_handle: Handle<Prefab<EnemyPrefab>>,
) -> GameplayHandles {
    let background_sprite_handle = load_sprite_sheet(world, "backgrounds", progress_counter);
    let overlay_sprite_handle = load_sprite_sheet(world, "transition", progress_counter);
    let glass_sprite_handle = load_sprite_sheet(world, "glass_shards", progress_counter);
    let enemy_sprites_handle = load_sprite_sheet(world, "enemy_sprites", progress_counter);
    let player_sprites_handle = load_sprite_sheet(world, "player_sprites", progress_counter);

    GameplayHandles {
        background_sprite_handle,
        overlay_sprite_handle,
        glass_sprite_handle,
        enemy_sprites_handle,
        enemy_prefab_handle,
        flying_enemy_prefab_handle,
        player_prefab_handle,
        player_hyper_prefab_handle,
        boss_prefab_handle,
        player_sprites_handle,
    }
}

// Helper for loading a spritesheet into asset storage.
fn load_sprite_sheet(world: &mut World, name: &str, progress_counter: &mut ProgressCounter) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            format!("sprites/{}.png", name),
            ImageFormat::default(),
            progress_counter,
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("sprites/{}.ron", name),
        SpriteSheetFormat(texture_handle),
        // TODO: should be progress_counter here too but would require some
        // restructuring
        (),
        &sprite_sheet_store,
    )
}
