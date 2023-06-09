// Referenced: The official Bevy event example here:
// https://github.com/bevyengine/bevy/blob/5718a7e74cc9b93d1d0bed9123548222123151b3/examples/ecs/event.rs
// on Sun, 12 Mar 2023
//
// This illustrated how to register events to the app with `add_event::<EventType>()`.

use bevy::{
    prelude::*,
    ecs::system::Command,
};

use crate::{
    error::ArgumentParseError,
    assets,
    displacement::{
        DisplacementMaterial,
        CullMode,
        TextureOption,
    },
    grid::hex::*,
    subdivision,
};

pub use compute::OceanComputeImages;

const OCEAN_FLOOR_TEXTURE_SIDE_LENGTH_METERS: f32 = 5.0;

/// One of [`EarthPlugins`](crate::EarthPlugins),
/// this sets up and runs the compute shader that simulates the ocean
/// surface.
pub struct OceanPlugin;

mod compute;

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<DisplacementMaterial>::default())
            .add_plugin(compute::OceanComputePlugin)
            .add_startup_system(load_floor_material);
    }
}

/// Loads the ocean floor texture using the terrain material api so that it tiles.
fn load_floor_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images_to_repeat: ResMut<assets::RepeatSampleImageQueue>,
) {
    commands.insert_resource(OceanAssets {
        floor_material: assets::load_terrain_material(
            "sand",
            &asset_server,
            &mut materials,
            &mut images_to_repeat
        )
    })

}

/// A marker structure for an ocean entity
#[derive(Clone, Copy, Component, Default)]
pub struct Ocean;

/// This structure holds a reference to the ocean floor material
///
/// This is so tiles can obtain a handle when they're added.
#[derive(Resource, Clone, Debug)]
pub struct OceanAssets {
    floor_material: Handle<StandardMaterial>,   
}

/// An command to easily add an Ocean tile to the world
///
/// Each ocean tile contains a seafloor and a surface above it. The
/// resolution and transform of the hex tile can be adjusted using the
/// [`AddOcean::resolution`], and [`AddOcean::grid_position`] command
/// fields. Note that each ocean surface will use the same
/// underlying textures, so you cannot add more than one simulation at
/// one time. Still, you can add more than one ocean *tile* at once
/// and you can tile them however you would like.
///
/// # Panics
/// Note that your app must use [OceanPlugin] or this command
/// will panic.
pub struct AddOcean {
    /// How many times to subdivide the surface mesh, defualts to 8
    pub resolution: u8,

    /// The amplitude of waves generated by the compute module, defaults to 4.0
    pub wave_height: f32,

    /// The position of the floor mesh relative to the surface, defaults to 10.0
    pub depth: f32,

    /// The grid position at which to put this tile, defaults to the center tile
    pub grid_position: GridVec,
}

impl AddOcean {
    /// This function builds the subdivided hex mesh for the ocean surface.
    fn create_surface_mesh(&self, world: &mut World) -> Handle<Mesh> {
        let size = world.resource::<Grid>().major_radius * 2.0;
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let subdivided_hexagon = subdivision::hexagon::new(self.resolution as u32, size, 1.0 / size)
            .expect("Couldn't build mesh for default ocean surface");

        meshes.add(subdivided_hexagon)
    }

    /// This function builds the hex mesh for the ocean floor.
    fn create_floor_mesh(&self, world: &mut World) -> Handle<Mesh> {
        let size = world.resource::<Grid>().major_radius * 2.0;
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let hexagon = subdivision::hexagon::new(
            0,
            size,
            1.0 / OCEAN_FLOOR_TEXTURE_SIDE_LENGTH_METERS
        ).expect("Couldn't build mesh for default ocean floor surface");
        
        meshes.add(hexagon)
    }

    /// This function pulls the ocean wave output from [`compute`]
    /// into a material that displaces the surface in the vertex
    /// shader.
    fn create_surface_material(&self, world: &mut World) -> Handle<DisplacementMaterial> {
        let images = world.get_resource::<OceanComputeImages>()
            .expect("no textures for the ocean, did you add the OceanPlugin before adding an ocean?")
            .clone();
        
        let mut displacement_materials = world.resource_mut::<Assets<DisplacementMaterial>>();

        displacement_materials.add(DisplacementMaterial {
            albedo: TextureOption::color_only(Color::rgba(0.5, 0.8, 1.0, 0.7)),
            displacement: Some(images.displacement),
            world_normal: Some(images.normal),
            alpha_mode: AlphaMode::Blend,
            cull_mode: CullMode::None,
            amplitude: self.wave_height,
            ..default()
        })
    }
}

impl Default for AddOcean {
    fn default() -> AddOcean {
        AddOcean {
            resolution: 9,
            wave_height: 4.0,
            depth: 10.0,
            grid_position: GridVec::ZERO,
        }
    }
}

impl TryFrom<Vec<&str>> for AddOcean {
    type Error = ArgumentParseError;

    fn try_from(args: Vec<&str>) -> Result<AddOcean, ArgumentParseError> {
        let mut args = args.into_iter();
        if Some("at") != args.next() {
            return Err(ArgumentParseError::ExpectedAt);
        }

        let grid_position = GridVec::try_from(args.collect::<Vec<&str>>())
            .map_err(|_| ArgumentParseError::GridVecParseError)?;

        Ok(AddOcean { grid_position, ..default() })
    }
}

/// An entity bundle for an ocean tile
///
/// This should always be added through the [`AddOcean`] command,
/// which will additionally add the surface and floor meshes it needs
/// to look right.
#[derive(Default, Bundle)]
struct OceanBundle {
    marker: Ocean,
    tile: Tile,
    spatial: SpatialBundle,
}

// Based on the Command implementation example in the bevy documention.
impl Command for AddOcean {
    fn write(self, world: &mut World) {
        let grid = world.get_resource::<Grid>()
            .expect("Cannot add an ocean tile without a grid!");
        let transform = Transform::from_translation(grid.to_world_position(self.grid_position));
        
        let surface_mesh = self.create_surface_mesh(world);
        let surface_material = self.create_surface_material(world);
        let floor_mesh = self.create_floor_mesh(world);
        let floor_material = world.resource::<OceanAssets>().floor_material.clone();

        world
            .spawn(OceanBundle{
                spatial: SpatialBundle { transform, ..default() },
                tile: Tile { grid_position: self.grid_position, elevation: -self.depth },
                ..default()
            })
            .insert(Name::new("Ocean Tile"))
            .with_children(|builder| {
                builder.spawn(MaterialMeshBundle {        
                    mesh: surface_mesh,
                    material: surface_material,
                    ..default()
                }).insert(Name::new("Ocean Surface"));

                builder.spawn(MaterialMeshBundle {
                    mesh: floor_mesh,
                    material: floor_material,
                    transform: Transform::from_translation(Vec3::Z * -self.depth),
                    ..default()
                }).insert(Name::new("Ocean Floor"));
            });
    }
}
