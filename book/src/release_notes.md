# Release Notes
## Features

This is our first release so we're summarizing all of our main features below.

- The earth library allows one to easily add three different "biome tiles" to a hex grid using the Bevy game engine.
  - The city tiles feature seven different layouts which are
    selectable through the appropriate add command.
  - The ocean tile uses a gerstner-wave on GPU approach to creating a
    fairly realistic ocean surface, and makes the surface
    displacement/normal maps available through the OceanComputeImages
    structure. Additional groundwork has been laid for a more
    statistical FFT approach (though some of this groundwork has yet
    to be ported from our older repo).
  - The forest tile features randomly arranged custom assets for a
    natural look. There is also a built-in level-of-detail system for
    the trees, though it does not always successfully render lower
    levels of detail (see known issues below).
- There is also an editor with a small console that you may use to
  mutate the world at runtime, generate environments and save relevant
  seeds for such procedural generation.

We hope you enjoy using the earth crate as much as we enjoyed making
it!

## Known Issues

There are a few known issues in our current implementation.

- The level of detail (LOD) system sometimes causes meshes at lower
  detail levels to stop rendering. Since large trees are the only
  asset that has an LOD, the forest environment is the only one
  affected by this bug.
- During console-based map generation, the simulation may hang briefly
  and warnings may be emitted about missing entities. We have not
  observed any functional loss from these warnings.
- Certain tiles (such as city tiles) may not lay out their contents
  appropriately when using a hex grid of a different size.

