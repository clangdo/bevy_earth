# Bevy Earth Environments

As part of a larger multi-team initiative to build simulations with
game development tools, this project realizes Earth-like environments
which are easy to adapt into any existing game or simulation using the
[Bevy Game Engine](https://bevyengine.org/).

## Usage
### **IMPORTANT**
**Any simulation provided by these environments is simple and
game-like.** While this crate can be used to get a basic idea, or a
simplified visualization of how a vehicle might move through a
relatively flat environment, **testing in more specialized simulation
systems and with real prototypes is key for any real world vehicle or
other safety-critical system.**

### Dependencies
The project requires an installation of
[`Rust`](https://www.rust-lang.org/) and it's accompanying package
tool `cargo`. In addition, it requires the [necessary
dependencies](https://bevyengine.org/learn/book/getting-started/setup/#install-os-dependencies)
for [Bevy](https://bevyengine.org/).

### Running
Once you have downloaded the dependencies, the project can be run like
any other Rust project. Simply download and run the `earth` binary
with `cargo run`, or you can run one of the `earth` examples with
`cargo run --example <name>`.

### Features
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
#### `add`
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

#### `clear`
This removes all tiles on the grid, it is currently the only way of removing tiles.

#### `generate`
This generates a 5-radius (49-tile) world out of seven random 7-tile
biomes. Note that this command may hang the simulation for a second
or two, and generate some harmless warnings, but it should work fine.

#### `save`
This command saves the *seed*—not the actual world—to a file given in
`[filename]` or to `./seed` if no filename is given. You can only save
a world created with `generate` this way, you cannot yet save manually
specified tile layouts.

#### `load`
This command loads the seed in the given filename (or `./seed` if no
filename is given). A `generate` command will generate the same
arrangement of biomes as the generation before that seed was saved.

#### `exit`
This command exits. It also has aliases `quit` and `q`.

## Authors
This project was implemented by [Caelan](https://github.com/clangdo)
[Chase Parker](https://github.com/alanparkerc), and
[BrandiCook](https://github.com/BrandiCook) as our senior project for
CS 46[1-3] at Oregon State University's undergraduate program. We
worked on this project under the excellent guidance of Dr. Chris
Patton of Patton Dynamics, LLC.

## Thank Yous
Many thanks to our project sponsor Dr. Patton, without whom this would
not have been possible. We are also tremendously grateful to our
instructors at Oregon State University, who were ready to help and
guide every step of the way. Though the project was finished and
prepared for release by Caelan and Chase, we could not have come as
far without Brandi, who prototyped the initial natural environments
that evolved into the current forest environment. Thank you all.

## Licensing
The code in this repository is licensed under the MIT license (see
`LICENSE`) except where otherwise specified.

### Third Party Licensing
Not all resources in this repository are licensed as specified in
`LICENSE`. Some assets (such as fonts) in this repository were
created by third parties and are redistributed under their respective
licenses. Each such resource is packaged in a directory with the
applicable license for that resource. 

If there is no such license file exists alongside an asset, then the
asset is either original (and distributed under MIT), or it is
licensed under CC0 (it is in the public domain). Still, CC0 files are
generally marked as such in the same directory, and their original
source is also cited in the same way.

## Contributing
Please format your code appropriately and lint with clippy. Pull
requests will be reviewed when possible by a core member of the
current project team.

By contributing you agree to have your contribution distributed as
part of the project under an MIT license irrevocably and in
perpetuity. You further agree that this distribution may be in any
forms that make sense for the project maintainers.
