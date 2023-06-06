// City Library file

use std::f32::consts::{FRAC_PI_3, PI};

use bevy::{ecs::system::Command, gltf::Gltf, prelude::*};

use crate::{error, grid::hex::*, subdivision};

pub struct CityPlugin;
use crate::city::urban::CityObject;
use crate::city::urban::GroundTexture;
use crate::city::urban::SideTexture;

pub mod urban;

// marker struct
#[derive(Clone, Copy, Component, Default)]
pub struct City;

// nature bundle
#[derive(Bundle, Default)]
pub struct CityBundle {
    pub city: City,
    tile: Tile,
    pub ground: MaterialMeshBundle<StandardMaterial>,
}

// Internal Grid Size
const INT_GRID_SIZE: f32 = 5.0;

// Direction builds will move towards when being generated.
pub enum Direction {
    North,
    East,
    South,
    West,
}

// implement the build function (call the loading of textures and models)
impl Plugin for CityPlugin {
    fn build(&self, app: &mut App) {
        let city_startup_systems = (CityPlugin::load_textures, CityPlugin::load_models);
        app.add_startup_systems(city_startup_systems.in_base_set(StartupSet::PreStartup));
    }
}

// plugin for the city scene
impl CityPlugin {
    // load the ground textures for the street and sidewalk.
    fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
        let ground_texture = asset_server.load("textures/ground_street_texture.png");
        commands.insert_resource(GroundTexture(ground_texture));
        let sidewalk_texture = asset_server.load("textures/ground_cobble_texture.png");
        commands.insert_resource(SideTexture(sidewalk_texture));
    }

    // load each of the gltf models (all models in /models)
    fn load_models(assets: Res<AssetServer>) {
        for CityObject { name, .. } in urban::ASSETS {
            let _handle = assets.load::<Gltf, String>(format!("models/{}.glb", name));
        }
    }
}

// // AddCity is the command to add a city tile (bevy)
#[derive(Default)]
pub struct AddCity {
    pub layout: i32,
    pub grid_position: GridVec,
}

// implement the command AddCity
impl AddCity {
    // traits specify world
    // create the sidewalk mesh
    fn create_side_ground(&self, world: &mut World) -> Handle<Mesh> {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        meshes.add(Mesh::from(shape::Box {
            min_x: 0.0,
            max_x: 1.0 * INT_GRID_SIZE,
            min_y: 0.0,
            max_y: 1.0 * INT_GRID_SIZE,
            min_z: 0.0,
            max_z: 0.1 * INT_GRID_SIZE,
        }))
    }

    // create the skyscraper mesh (for each floor of the building)
    fn create_skyscraper(&self, world: &mut World) -> Handle<Mesh> {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        meshes.add(Mesh::from(shape::Box {
            min_x: 0.0,
            max_x: 0.01,
            min_y: 0.0,
            max_y: 0.01,
            min_z: 0.0,
            max_z: 0.01,
        }))
    }

    // create the material for the standard/default ground
    fn create_material(&self, world: &mut World) -> Handle<StandardMaterial> {
        let texture = world.resource::<GroundTexture>().0.clone(); // get the resource already inserted by plugin
        let mut material = world.resource_mut::<Assets<StandardMaterial>>(); // call add on to the collection to return assets
        material.add(urban::create_material(texture)) // return materials
    }

    // create the material for the sidewalk
    fn create_side_material(&self, world: &mut World) -> Handle<StandardMaterial> {
        let texture = world.resource::<SideTexture>().0.clone(); // get the resource already inserted by plugin
        let mut material = world.resource_mut::<Assets<StandardMaterial>>(); // call add on to the collection to return assets
        material.add(urban::create_material(texture)) // return materials
    }

    // create the material for the skyscraper floor
    fn create_sky_material(&self, world: &mut World) -> Handle<StandardMaterial> {
        let mut material = world.resource_mut::<Assets<StandardMaterial>>(); // call add on to the collection to return assets
        material.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..default()
        })
    }

    // create the mesh for the main base of the hexagonal tile
    fn create_floor_mesh(&self, world: &mut World) -> Handle<Mesh> {
        let size = world.resource::<Grid>().major_radius * 2.0;
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let hexagon = subdivision::hexagon::new(0, size, 1.0 / 50.0)
            .expect("Couldn't build mesh for default city floor surface");

        meshes.add(hexagon)
    }

    // function creates a skyscraper with the specified number of floors and xy coordinates.
    fn build_skyscraper(
        parent: &mut WorldChildBuilder,
        sky_mesh: Handle<Mesh>,
        sky_material: Handle<StandardMaterial>,
        floors: i32,
        xsky: f32,
        ysky: f32,
        c: Handle<Scene>,
    ) {
        let true_scale = 1.0 * INT_GRID_SIZE;
        // let scene_path = format!("models/low_poly_floor.glb#Scene0");
        parent
            .spawn(PbrBundle {
                mesh: sky_mesh,
                material: sky_material,
                transform: Transform::from_xyz(xsky, ysky, 0.0),
                ..default()
            })
            .with_children(|parent| {
                for f in 0..floors {
                    parent.spawn(SceneBundle {
                        scene: c.clone(),
                        transform: Transform::from_xyz(0.0, 0.0, (f as f32) * (true_scale * 1.6))
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3 {
                                x: 0.325 * INT_GRID_SIZE,
                                y: 0.325 * INT_GRID_SIZE,
                                z: 0.325 * INT_GRID_SIZE,
                            }),
                        ..default()
                    });
                }
            });
    }

    // Need to add warning for improper direction
    // creates a strip of sidewalk in the direction specified in the parameters on the specifed xy coordinates
    fn build_sidewalk(
        parent: &mut WorldChildBuilder,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        xside: f32,
        yside: f32,
        amount: i32,
        direction: Direction,
    ) {
        let mut temp_x = xside;
        let mut temp_y = yside;

        // sidewalk_scale is the standard size of a sidewalk tile
        let sidewalk_scale = 1.0 * INT_GRID_SIZE;
        // for loop through amount parameter in the direction specified with the number in the parameters.
        for i in 0..amount {
            match direction {
                Direction::North => temp_y = (sidewalk_scale * i as f32) + yside,
                Direction::East => temp_x = (sidewalk_scale * i as f32) + xside,
                Direction::South => temp_y = -(sidewalk_scale * i as f32) + yside,
                Direction::West => temp_x = -(sidewalk_scale * i as f32) + xside,
            };
            parent
                .spawn(PbrBundle {
                    transform: Transform {
                        translation: (Vec3::new(temp_x, temp_y, 0.0)),
                        ..default()
                    },
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                })
                .insert(Name::new("Sidewalk"));
        }
    }
}

impl TryFrom<Vec<&str>> for AddCity {
    type Error = error::ArgumentParseError;

    fn try_from(args: Vec<&str>) -> Result<AddCity, Self::Error> {
        let mut args = args.into_iter();

        if Some("layout") != args.next() {
            return Err(Self::Error::ExpectedLayout);
        }

        let layout = args
            .next()
            .ok_or(Self::Error::ExpectedLayout)?
            .parse::<i32>()
            .map_err(|_| Self::Error::LayoutParseError)?;

        if Some("at") != args.next() {
            return Err(Self::Error::ExpectedAt);
        }

        let grid_position = GridVec::try_from(args.collect::<Vec<&str>>())
            .map_err(|_| Self::Error::GridVecParseError)?;

        Ok(AddCity {
            grid_position,
            layout,
        })
    }
}
// implements the functionality for functions
impl Command for AddCity {
    fn write(self, world: &mut World) {
        // check for the grid
        let grid = world
            .get_resource::<Grid>()
            .expect("Cannot add an city tile without a grid!");

        // create the transform for the initial surface
        let surface_transform =
            Transform::from_translation(grid.to_world_position(self.grid_position));

        // create the base tile hex material
        let hex_material = self.create_material(world);
        let hex_mesh = self.create_floor_mesh(world);

        // create the sidewalk mesh handle and the sidewalk material handle
        let side_mesh = self.create_side_ground(world);
        let side_material = self.create_side_material(world);

        // create the skyscraper mesh handle and the skyscraper material handle
        let sky_mesh_handle = self.create_skyscraper(world);
        let sky_mat = self.create_sky_material(world);

        // incorporate the asset server
        let asset_server = world.resource::<AssetServer>();

        // load the floor glb file with the asset server
        let ftest = asset_server.load("models/floor.glb#Scene0");

        let area_map_1 = [
            [18.0, 24.0],
            [-18.0, 24.0],
            [18.0, 12.0],
            [-18.0, 12.0],
            [18.0, 0.0],
            [-31.0, 0.0],
            [31.0, 0.0],
            [-18.0, 0.0],
            [18.0, -12.0],
            [-18.0, -12.0],
            [-18.0, -24.0],
            [18.0, -24.0],
        ];

        // self layout 0
        // Basic through street with buildings running adjacent
        match self.layout {
            0 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform,
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            1.5 * INT_GRID_SIZE,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            17,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            17,
                            Direction::North,
                        );

                        let floors = 3;
                        for q in area_map_1 {
                            AddCity::build_skyscraper(
                                parent,
                                sky_mesh_handle.clone(),
                                sky_mat.clone(),
                                floors,
                                q[0] * INT_GRID_SIZE / 4.0,
                                q[1] * INT_GRID_SIZE / 4.0,
                                ftest.clone(),
                            );
                        }
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Buildings"));
            }
            // self layout 1
            //
            1 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform,
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            4,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            4,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            4,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            4,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );

                        let floors = 7;

                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            -18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Cross Section"));
            }
            // self layout 2
            2 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform,
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -24.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            12,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -24.0 * INT_GRID_SIZE / 4.0,
                            -28.0 * INT_GRID_SIZE / 4.0,
                            12,
                            Direction::East,
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Empty"));
            }
            // self layout 3
            3 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform
                            .with_rotation(Quat::from_rotation_z(FRAC_PI_3)),
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            32.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            32.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            32.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            32.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );

                        let floors = 7;

                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            -18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Cross Section"));
            }
            // self layout 4
            4 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform
                            .with_rotation(Quat::from_rotation_z(FRAC_PI_3)),
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -34.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            32.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            32.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::South,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            32.0 * INT_GRID_SIZE / 4.0,
                            6.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            32.0 * INT_GRID_SIZE / 4.0,
                            -10.0 * INT_GRID_SIZE / 4.0,
                            7,
                            Direction::West,
                        );

                        let floors = 7;

                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                        AddCity::build_skyscraper(
                            parent,
                            sky_mesh_handle.clone(),
                            sky_mat.clone(),
                            floors,
                            -18.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            ftest.clone(),
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Cross Section"));
            }
            // self layout 5
            5 => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform.with_rotation(Quat::from_rotation_z(PI)),
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -24.0 * INT_GRID_SIZE / 4.0,
                            24.0 * INT_GRID_SIZE / 4.0,
                            12,
                            Direction::East,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -24.0 * INT_GRID_SIZE / 4.0,
                            -28.0 * INT_GRID_SIZE / 4.0,
                            12,
                            Direction::East,
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Empty"));
            }
            // self layout null
            // default case: through sidewalk
            _ => {
                world
                    .spawn(PbrBundle {
                        mesh: hex_mesh,
                        material: hex_material,
                        transform: surface_transform,
                        ..default()
                    })
                    .with_children(|parent| {
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            6.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            17,
                            Direction::North,
                        );
                        AddCity::build_sidewalk(
                            parent,
                            side_mesh.clone(),
                            side_material.clone(),
                            -10.0 * INT_GRID_SIZE / 4.0,
                            -34.0 * INT_GRID_SIZE / 4.0,
                            17,
                            Direction::North,
                        );
                    })
                    .insert(Tile {
                        grid_position: self.grid_position,
                        elevation: 0.0,
                    })
                    .insert(Name::new("City - Path"));
            },
        }
    }
}
