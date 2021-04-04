I haven't found a good way to modify prefabs at runtime or autodetect
hidpi and change the pixel values in amethyst UI elements.

Example: a 1920x1080 image will be too small on a retina display, where
the window is renderered as 2880 x 1710.

If the UI elements in the .ron files in this folder are built for retina
displays, they'll be too large on normal displays, or vice versa.
If I'm doing this all wrong -- please tell me!

Until then, I have copy/paste configs with values for different monitors.
