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
