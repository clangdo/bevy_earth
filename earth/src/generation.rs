use bevy::{
    ecs::system::Command,
    prelude::*,
};

use crate::{
    city,
    ocean,
    nature,
    grid::hex::GridVec,
    rng::{
        EarthRng,
        LastGenerationSeed,
    },
};

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GenerationRequested(false))
            .add_system(generate);
    }
}

/// This is a flag struct for procedural generation
///
/// When it is true, the procedural generation system clears the world
/// and creates one map, changing this structure to false in the
/// process.
#[derive(Resource, Clone, Copy, Debug)]
struct GenerationRequested(bool);

fn generate(mut commands: Commands, rng: Res<EarthRng>, mut requested: ResMut<GenerationRequested>) {
    if !requested.0 { return; }

    requested.0 = false;

    let rng_lock = rng.0.lock().expect("unable to lock rng for world generation");

    commands.insert_resource(LastGenerationSeed(rng_lock.get_seed()));
    
    // Generate a random array of seed values
    let proc_map = std::iter::repeat_with(|| rng_lock.usize(1..=3)).take(7);

    let zero_position: GridVec = GridVec::ZERO; // Initial position is (0, 0)
    let positions = [
        zero_position,
        zero_position + (GridVec::SOUTHEAST * 2) + GridVec::SOUTH,
        zero_position + (GridVec::NORTHEAST * 3) + GridVec::SOUTH,
        zero_position + (GridVec::NORTH * 2) + GridVec::NORTHEAST,
        zero_position + (GridVec::SOUTH * 2) + GridVec::SOUTHWEST,
        zero_position + (GridVec::NORTHWEST * 2) + GridVec::NORTH,
        zero_position + (GridVec::SOUTHWEST * 3) + GridVec::NORTH,
    ];
    
    for (i, seed) in proc_map.enumerate() {
        match seed {
            1 => add_city_biome(&mut commands, positions[i]),
            2 => add_ocean_biome(&mut commands, positions[i]),
            3 => add_forest_biome(&mut commands, positions[i]),
            _ => panic!("unhandled biome type encountered during generation!"),
        }
    }
}

fn biome_tile_positions(around: GridVec) -> impl Iterator<Item = GridVec> {
    std::iter::once(around).chain(around.neighbors())
}

fn add_city_biome(commands: &mut Commands, around: GridVec) {
    let layouts = [1, 0, 0, 5, 5, 5, 5];
    for (grid_position, layout) in std::iter::zip(biome_tile_positions(around), layouts) {
        commands.add(city::AddCity {
            grid_position,
            layout,
        });
    }
}

fn add_ocean_biome(commands: &mut Commands, around: GridVec) {
    for grid_position in biome_tile_positions(around) {
        commands.add(ocean::AddOcean {
            grid_position,
            ..default()
        });
    }
}

fn add_forest_biome(commands: &mut Commands, around: GridVec) {
    for grid_position in biome_tile_positions(around) {
        commands.add(nature::AddForest {
            grid_position,
        });
    }
}

pub struct ScheduleGenerate;

impl Command for ScheduleGenerate {
    fn write(self, world: &mut World) {
        world.resource_mut::<GenerationRequested>().0 = true;
    }
}
