## Benitron 3000

A work-in-progress robotron style game made using https://amethyst.rs.

### Quick build

```bash
git clone https://github.com/ericrasmussen/benitron3000.git
cd benitron3000
cargo run
```

### Gameplay notes

This game is pre-pre-pre-alpha and is being developed as a learning exercise.

The controls are wasd for movement and the arrow keys for firing lasers.

The game supports many experimental features, including being able to walk offscreen for infinity, perhaps never to return. Several of these highly advanced gameplay mechanics and features may be adjusted as development progresses, with the goal of eventually resembling an actual game.

### Additional build instructions

The information below comes from the https://github.com/amethyst/amethyst-starter-2d README:

#### For Mac Users

This starter uses vulkan as a renderer by default. You'll want to change the backend to use `metal`, which can be done by opening the `Cargo.toml` file and changing

```toml
[features]
default = ["vulkan"]
```

to

```toml
[features]
default = ["metal"]
```

If using OSX and Metal you will require full XCode installed from the Appstore in order to compile metal shaders.
After install you may be required to run this command `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer` [reference gfx-rs issue](https://github.com/gfx-rs/gfx/issues/2472)

#### For Linux Users

You might need to install some dependencies. Please refer to [this section](https://github.com/amethyst/amethyst#dependencies) of the README for more details.

