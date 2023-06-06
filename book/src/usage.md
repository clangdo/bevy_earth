# Program Usage

## **IMPORTANT**
**Any simulation provided by these environments is simple and
game-like.** While this crate can be used to get a basic idea, or a
simplified visualization of how a vehicle might move through a
relatively flat environment, **testing in more specialized simulation
systems and with real prototypes is key for any real world vehicle or
other safety-critical system.**

## Dependencies
Building the project requires an installation of
[Rust](https://www.rust-lang.org/) and it's accompanying package tool
`cargo`. In addition, it requires the [necessary
dependencies](https://bevyengine.org/learn/book/getting-started/setup/#install-os-dependencies)
for [Bevy](https://bevyengine.org/). Other dependencies will be
fetched and compiled automatically by `cargo` when you run the
project.

If you have a prebuilt binary, these development libraries are not
necessary, and you will likely be able to run it without installing
anything[^linux_desktop].

[^linux_desktop] If you are running a UNIX machine, you should ensure
that you are not trying to run a wayland-specific build in an X11 unix
desktop environment. Conversely, if you are running wayland and would
like to run an X11 build, ensure you have Xwayland set up properly.

## Running
Once you have downloaded the dependencies, the project can be run like
any other Rust project. Simply download and run the `earth` binary
with `cargo run`, or you can run one of the `earth` examples with
`cargo run --example <name>`.

## Features
The main binary should present you with a blank screen by
default. This is an empty world which can be manipulated through use
of the console. You can open the console with `` ` `` (grave/tilde) which
will allow you to run commands.

The supported commands are listed below.
```
add <biome setup> at <location>
clear
generate
save [filename]
load [filename]
exit
```

### `add`
The `<biome setup>` field can be `ocean`, `forest` or `city layout N`
where `N` is an integer from 0-6 inclusive. Experiment with the
different layouts to see what they all look like!

The `<location>` field specifies a hex tile coordinate at which to add
the biome. Each coordinate is in the form `vector [vector]`, where
each vector is `<int><direction>` and directions can be
`n|ne|nw|s|se|sw` for north, northeast, northwest, etc. You can
specify any tile on an infinite hex grid this way, but note that tiles
very far from the origin may be processed incorrectly due to floating
point error.

### `clear`
This removes all tiles on the grid, it is currently the only way of removing tiles.

### `generate`
This generates a 5-radius (49-tile) world out of seven random 7-tile
biomes. Note that this command may hang the simulation for a second
or two, and generate some harmless warnings, but it should work fine.

### `save`
This command saves the *seed*—not the actual world—to a file given in
`[filename]` or to `./seed` if no filename is given. You can only save
a world created with `generate` this way, you cannot yet save manually
specified tile layouts.

### `load`
This command loads the seed in the given filename (or `./seed` if no
filename is given). A `generate` command will generate the same
arrangement of biomes as the generation before that seed was saved.

### `exit`
This command exits the program. It also has aliases `quit` and `q`.
