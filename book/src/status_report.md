# Release Notes

## Features

TODO

## Known Issues: 

There are a few known issues in our current implementation. 

- The level of detail (LOD) system sometimes causes meshes at lower
  detail levels to stop rendering. Since trees are the only asset that has
- During console-based map generation, the simulation may hang briefly
  and warnings may be emitted about missing entities. We have not
  observed any functional loss from these warnings.
- Certain tiles (such as city tiles) may not lay out their contents
  appropriately when using a hex grid of a different size.
- Current size testing shows that different tile sizes can be ran, but doesn't
  display them for the user at this time.
- The gen.rs test file is currently not working. Supposed to implement 5 different
  instances of the app, but fails to do so. May be too many.
- Giving a warning from the asset_server and bevy_render that certain assets aren't
  loading. May only be a problem on certain systems.
