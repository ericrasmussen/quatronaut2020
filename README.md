## Benitron 3000

A work-in-progress robotron style game made using https://amethyst.rs.

### Quick Build and Run

```bash
git clone https://github.com/ericrasmussen/benitron3000.git
cd benitron3000
cargo run --features "metal"    # if on macOS
cargo run --features "vulkan"   # if on Linux or win32
```

### Gameplay notes

This game is pre-pre-pre-alpha and is being developed as a learning exercise.

The controls are wasd for movement and the arrow keys for firing lasers.

The game supports many experimental features, including being able to walk offscreen for infinity, perhaps never to return. Several of these highly advanced gameplay mechanics and features may be adjusted as development progresses, with the goal of eventually resembling an actual game.

### Additional build instructions

The information below comes from the https://github.com/amethyst/amethyst-starter-2d README:

#### For Linux Users

You might need to install some dependencies. Please refer to [this section](https://github.com/amethyst/amethyst#dependencies) of the README for more details.

