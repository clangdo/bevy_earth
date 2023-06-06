use bevy::{ecs::system::Command, gltf::Gltf, prelude::*};

use crate::{assets, error::ArgumentParseError, grid::hex::*, rng::EarthRng, subdivision};

use super::{create_spawn_tasks, NaturalObject};

const FOREST_FLOOR_TEXTURE_SIDE_LENGTH_METERS: f32 = 3.0;

/// A simple array enumerating the natural gltf scenes and thier other statistics
pub const ASSETS: [NaturalObject; 5] = [
    NaturalObject {
        name: "pine",
        radius: 4.0,
        count: 80,
        cull_distance: f32::INFINITY, // Never cull trees
        extra_lods: 1,
        lod_distance_step: 200.0,
    },
    NaturalObject {
        name: "pine_small",
        radius: 1.0,
        count: 50,
        cull_distance: 300.0,
        extra_lods: 0,
        lod_distance_step: 20.0,
    },
    NaturalObject {
        name: "pine_stump",
        radius: 0.5,
        count: 10,
        cull_distance: 200.0,
        extra_lods: 0,
        lod_distance_step: 20.0,
    },
    NaturalObject {
        name: "boulder_1",
        radius: 3.0,
        count: 8,
        cull_distance: 500.0,
        extra_lods: 0,
        lod_distance_step: 20.0,
    },
    NaturalObject {
        name: "boulder_2",
        radius: 3.0,
        count: 14,
        cull_distance: 500.0,
        extra_lods: 0,
        lod_distance_step: 20.0,
    },
];

/// Load the models for the forest environment, including their LODs, if any.
pub fn load_models(assets: Res<AssetServer>) {
    for NaturalObject {
        name, extra_lods, ..
    } in ASSETS
    {
        let _ = assets.load::<Gltf, String>(format!("models/{name}.glb"));

        // Also load lods if there are any
        for lod_index in 1..=extra_lods {
            let _ = assets.load::<Gltf, String>(format!("models/{name}_l{lod_index}.glb"));
        }
    }
}

/// Load the material for the forest floor and all necessary textures
pub fn load_ground_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images_to_repeat: ResMut<assets::RepeatSampleImageQueue>,
) {
    commands.insert_resource(ForestFloorMaterial(assets::load_terrain_material(
        "coniferous_forest_floor",
        &asset_server,
        &mut materials,
        &mut images_to_repeat,
    )));
}

/// A marker structure for the currently supported natural environment
#[derive(Clone, Copy, Component, Default)]
pub struct Forest;

/// A bundle for a forest tile
#[derive(Bundle, Default)]
pub struct ForestBundle {
    pub marker: Forest,
    pub tile: Tile,
    pub ground: MaterialMeshBundle<StandardMaterial>,
}

/// This command to adds a single forest hex tile to a hex grid.
///
/// Note that this requires a [`Grid`] resource in the world to work.
pub struct AddForest {
    pub grid_position: GridVec,
}

impl Default for AddForest {
    fn default() -> AddForest {
        AddForest {
            grid_position: GridVec::ZERO,
        }
    }
}

impl TryFrom<Vec<&str>> for AddForest {
    type Error = ArgumentParseError;

    fn try_from(args: Vec<&str>) -> Result<AddForest, ArgumentParseError> {
        let mut args = args.into_iter();
        if Some("at") != args.next() {
            return Err(ArgumentParseError::ExpectedAt);
        }

        let grid_position = GridVec::try_from(args.collect::<Vec<&str>>())
            .map_err(|_| ArgumentParseError::GridVecParseError)?;

        Ok(AddForest { grid_position })
    }
}

fn create_ground_mesh(world: &mut World) -> Handle<Mesh> {
    let size = world.resource::<Grid>().major_radius * 2.0;
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let subdivided_hexagon =
        subdivision::hexagon::new(0, size, 1.0 / FOREST_FLOOR_TEXTURE_SIDE_LENGTH_METERS)
            .expect("Couldn't build mesh for nature tile");
    meshes.add(subdivided_hexagon)
}

/// A resource holding a handle to the forest floor material
#[derive(Clone, Debug, Resource)]
pub struct ForestFloorMaterial(pub Handle<StandardMaterial>);

impl Command for AddForest {
    fn write(self, world: &mut World) {
        let grid = world
            .get_resource::<Grid>()
            .expect("Cannot add a nature tile without a grid!");
        let surface_transform =
            Transform::from_translation(grid.to_world_position(self.grid_position));

        let ground_material = world.resource::<ForestFloorMaterial>().0.clone();
        let ground_mesh = create_ground_mesh(world);

        let rng_guard = world.resource::<EarthRng>().0.lock().unwrap().clone();

        let spawn_tasks = create_spawn_tasks(world.resource::<AssetServer>(), ASSETS.iter());

        world
            .spawn(ForestBundle {
                tile: Tile {
                    grid_position: self.grid_position,
                    elevation: 0.0,
                },
                ground: PbrBundle {
                    mesh: ground_mesh,
                    material: ground_material,
                    transform: surface_transform,
                    ..default()
                },
                ..default()
            })
            .with_children(|builder| {
                let mut colliders = Vec::new();
                for task in spawn_tasks {
                    task.attempt(&mut colliders, &rng_guard, builder);
                }
            })
            .insert(Name::new("Forest Tile"));
    }
}

// ensures capitalization is consistent with named objects (for the world inspector)
pub fn to_title_case<S: Into<String>>(train_case: S) -> String {
    let train_case: String = train_case.into();
    let mut title_case = String::new();

    for word in train_case.split('_') {
        if word.is_empty() {
            continue;
        }

        let (first_character, rest) = word.split_at(1);

        // Capitalize first character and push it into our result
        // string with the rest of this word
        title_case.push_str(first_character.to_uppercase().as_str());
        title_case.push_str(rest);
        title_case.push(' ')
    }

    // Remove the trailing space
    title_case.pop();
    title_case
}
