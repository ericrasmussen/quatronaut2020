## Quatronaut 2020

A work-in-progress robotron style game made using https://amethyst.rs.

### Quick Build and Run

```bash
git clone https://github.com/ericrasmussen/quatronaut2020.git
cd quatronaut2020
cargo xtask run
```

### Gameplay notes

This game is for fun and mostly exists to show off rust and amethyst. I am not an
accomplished game developer, and this is being developed as a learning exercise.
But, it works! Kind of! Feel free to use any of the code for your own rust and amethyst
games.

The controls are wasd for movement and the arrow keys for firing lasers.

### Triple A Features

- immersive sound effects
- a rockin' chiptunes track
- polygonal action with more than one shape
- it's sort of like Robotron but not
- surprise transition and advanced camera work that changes EVERYTHING
- it's technically possible to win. I did it once
- working menu screens! kind of! ...sometimes
- will be shipped ahead of schedule with lots of bugs

### Docs

I want this game to be as useful as possible to other rust or amethyst game
developers, so be sure to use one of:

# macos builds:
`cargo doc --features metal`

# other builds:
`cargo doc --features vulkan`

The docs are written more like a design decisions document or development blog
(as opposed to typical API docs). This is to make it clear when alternative approaches
exist or when things are complicated and ideally would be cleaned up (presumably I'm
lacking the time or motivation to do said cleaning).


### Secret cheat codes

OK, there's only one secret. Press 'g' during gameplay to toggle HYPER IMMORTAL mode.
If you start a new level in this mode, you'll have increased movement speed and fire rate.

There's also 'p' to pause without a menu, but if you forget to press 'p' a second time to
unpause, you may become stuck in a timeless land for a brief eternity or two.

### Linux dependencies

See https://github.com/amethyst/amethyst#dependencies for extra system dependencies needed
by linux distros.

### Assets

Special thanks to Joel Steudler at https://joelsteudler.itch.io/ for making every art and sound asset in the game. If you
are a game developer, check out his site! His asset packs are top-notch and have appeared regularly on
Humble Bundle and https://gameassetbundle.com/


##### Font

The primary font used in the game is the free font "Font Over" (https://fontesk.com/font-over-typeface/) by Ilya Zakharov.
Check out his other work at https://www.behance.net/zielgraphic


### Known issues

Things I'll address one day when I have infinite time and resources:

* add tests (I started and then stopped)
* the UI is flaky and I'm not sure if it's my code or amethyst UI or both
* sometimes projectiles don't get cleaned up before the level changes
* there are tons of gameplay ideas we had and just didn't get around to

### Copyright Information

Quatronaut 2020 name and logo is Copyright 2020 Eric Rasmussen and Joel Steudler.
