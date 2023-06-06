//! This crate facilitates the easy setup of earth-like environments
//! for simulation using the dynamics libraries in the same
//! repository.
//!
//! **IMPORTANT: The simulation provided by these environments is, as
//! of this writing, simple and game-like.** While this crate can be
//! used to get a basic idea, or a simplified visualization of how a
//! wheeled vehicle might move through a relatively flat environment,
//! **testing in more specialized simulation systems and with real
//! prototypes is key for any real world vehicle or other
//! safety-critical system.**
//!
//! This crate uses [`bevy`] as its underlying engine, make sure to
//! read up on the bevy basics before using this crate.
//!
//! The basic setup for this crate involves adding
//! [`EarthPlugins`] to the app and then adding individual tiles
//! using [`grid::hex::GridVec`] passed to the various add commands.
//! The add commands are [`ocean::AddOcean`] [`city::AddCity`], and
//! [`nature::AddForest`].

/// This module provides the urban environment
#[cfg(feature = "city")]
pub mod city;

/// This module provides the forest environment
#[cfg(feature = "nature")]
pub mod nature;

/// This module provides the ocean environment
#[cfg(feature = "ocean")]
pub mod ocean;

/// This module provides facilities for building a hex-grid based
/// world out of the [`city`], [`nature`], and [`ocean`] modular
/// environments.
pub mod grid;

/// The prelude module exposes the public API of the crate in a
/// convenient package.
pub mod prelude {
    pub use crate::EarthPlugins;
    pub use crate::ocean;
    pub use crate::nature;
    pub use crate::city;
    pub use crate::grid;
    pub use crate::query_ground_height;
    pub use crate::query_ground_height_simple;
    pub use crate::query_ground_normal;
    pub use crate::ClearGrid;
    pub use crate::rng::SaveSeed;
    pub use crate::generation::ScheduleGenerate;
}

/// This module facilitates the loading of certain assets.
///
/// Currently it simply helps load ground textures that need to tile.
mod assets;

/// This module sets a suitable background color for an earth like sky.
mod sky;

/// This module provides subdivided primitives
///
/// In particular it provides subdivided triangle and hexagon
/// meshes. Note that currently the hexagon mesh doubles certain
/// vertices (at the edge of each triangle). It also doesn't connect
/// the last edge with the first at all, so one of the sets of edges
/// between adjacent triangles is not bridged.
mod subdivision;

/// This module provides a version of the bevy standard material that
/// displaces vertices in a vertex shader.
///
/// The material includes the ability to take a world-space normal map
/// as well, which will be used to provide the *vertex normals*, not
/// the fragment normals.
mod displacement;

/// This module provides simple LOD functionality. Note that the
/// functionality is tuned for *static* meshes, but could be adjusted
/// for scenes that move as well (at the cost of some CPU time).
mod lod;

/// This facilitates a global rng resource shared between the
/// environments for deterministic procedural generation.
pub mod rng;

/// This facilitates procedural generation of a map
pub mod generation;

use bevy::{
    prelude::*,
    app::PluginGroupBuilder,
    ecs::system::Command,
};

/// This module includes the editor argument error for the crate
///
/// In future it could contain other general crate errors as well.
mod error;

/// These plugins load assets and simulate the earth like
/// environments. They are required in order to use this crate.
///
/// ```rust
/// use earth::prelude::*;
/// use bevy::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(EarthPlugins)
/// //  ...other setup
/// #   ;
/// ```
pub struct EarthPlugins;

impl PluginGroup for EarthPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(rng::RngPlugin)
            .add(assets::AssetPlugin)
            .add(lod::LodPlugin)
            .add(grid::hex::GridPlugin::default())
            .add(sky::SkyPlugin)
            .add(ocean::OceanPlugin)
            .add(nature::NaturePlugin)
            .add(city::CityPlugin)
            .add(generation::GenerationPlugin)
    }
}

/// This function the ground elevation at a given world
/// position.
///
/// You can get the required grid as a resource after adding
/// [`EarthPlugins`]. The tiles should be obtained by query for
/// `(Entity, &Tile)`. Note that this API will change if more detailed
/// elevation data is needed.
pub fn query_ground_height<'a, I>(at: Vec2, grid: &grid::hex::Grid, mut tiles: I) -> f32 where
    I: Iterator<Item = (Entity, Ref<'a, grid::hex::Tile>)>,
{
    let grid_coordinate = grid.to_grid_coordinate(at);
    let tile_id = grid.tiles.get(&grid_coordinate);

    match tile_id {
        Some(id) => {
            tiles.find(|(entity, _)| entity == id)
                .map(|(_, tile)| tile.elevation )
                .unwrap_or(f32::NEG_INFINITY)
        },
        None => f32::NEG_INFINITY
    }
}

/// This function is a temporary test function that always returns 0
pub fn query_ground_height_simple<'a, I>(_at: Vec2, _grid: &grid::hex::Grid, mut _tiles: I) -> f32 where
    I: Iterator<Item = (Entity, &'a grid::hex::Tile)>,
{
    0.0
}

/// This allows one to get the ground normal at a given world position
///
/// Note that currently the normal is always straight up.
pub fn query_ground_normal<'a, I>(_at: Vec2, _grid: &grid::hex::Grid, _tiles: I) -> Vec3 where
    I: Iterator<Item = (Entity, &'a grid::hex::Tile)>,
{
    Vec3::Z
}

pub struct ClearGrid;

impl Command for ClearGrid {
    fn write(self, world: &mut World) {
        use grid::hex::Tile;
        let mut tile_entities = world.query::<(&Tile, Entity)>();
        let tile_entities: Vec<(grid::hex::GridVec, Entity)> = tile_entities
            .iter(world)
            .map(|(tile, entity)| (tile.grid_position, entity))
            .collect();


        for (tile_position, entity) in tile_entities {
            bevy::hierarchy::despawn_with_children_recursive(world, entity);
            let mut grid = world.resource_mut::<grid::hex::Grid>();
            grid.tiles.remove(&tile_position).expect("uncached tile removed from world");
        }
    }
}
