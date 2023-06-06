use bevy::{
    utils::HashMap,
    prelude::*,
};

use std::{
    num::ParseIntError,
    f32::consts::*,
};

// References:
// - https://www.redblobgames.com/grids/hexagons/
// - Li, Xiangguo. Storage and addressing scheme for practical hexagonal image processing. https://doi.org/10.1117/1.JEI.22.1.010502

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.866_025_4;

/// This structure represents a hex grid
///
/// It contains the information needed to determine the geometry of a
/// given tile and makes the assumption that the grid is oriented with
/// vertices due east (+x) and west (-x) and edges north (+y) and
/// south (-y).
#[derive(Resource, Clone, Debug)]
pub struct Grid {
    /// The long dimension of all the hex tiles (vertex to opposite vertex)
    pub major_radius: f32,
    /// The origin of the grid in world coordinates
    pub origin: Vec3,
    /// A hash map for the tiles
    pub tiles: HashMap<GridVec, Entity>,
}

pub struct GridPlugin {
    /// The long dimension of all the hex tiles (vertex to opposite vertex)
    pub major_radius: f32,
    /// The origin of the grid in world coordinates
    pub origin: Vec3,
}

impl Default for GridPlugin {
    fn default() -> GridPlugin {
        let default_grid = Grid::default();
        GridPlugin {
            major_radius: default_grid.major_radius,
            origin: default_grid.origin,
        }
    }
}

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid {
            major_radius: self.major_radius,
            origin: self.origin,
            tiles: HashMap::new()
        }).add_system(cache_new_tiles);
    }
}


/// A component marking hex grid tiles
///
/// This also stores the *original* grid position that the tile was
/// spawned with. This grid position is not updated unless you (the
/// user of this library) update it yourself. It is not kept in sync
/// with the transform components in any way.
///
/// Note that currently updating this position will **invalidate the
/// tile cache** (stored as [`Grid::tiles`]) so it's not advisable. This
/// will be fixed in a future update.
///
/// Ideally, to change a tile's position one should update both this
/// and the tile's transform, or write a system to derive the
/// transforms from this position each update using
/// [`Grid::to_world_position`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Tile {
    
    pub grid_position: GridVec,
    /// A placeholder value, should be replaced by a collision mesh handle later
    pub elevation: f32,
}

/// Cache the tiles in the grid resource so they can be queried easily
fn cache_new_tiles(tiles: Query<(Ref<Tile>, Entity)>, mut grid: ResMut<Grid>) {
    let new_tiles = tiles
        .iter()
        .filter(|(tile, _)| tile.is_added())
        .map(|(tile, id)| (tile.grid_position, id));
    grid.tiles.extend(new_tiles);
}

impl Default for Grid {
    fn default() -> Grid {
        Grid {
            major_radius: 50.0,
            origin: Vec3::ZERO,
            tiles: HashMap::new(),
        }
    }
}

impl Grid {
    fn to_world_matrix(&self) -> Mat2 {
        let r = self.major_radius;
        // This is the basis for "axial" coordinates
        Mat2::from_cols(
            Vec2::new(1.5 * r, COS_FRAC_PI_6 * r),
            Vec2::new(0.0, 2.0 * COS_FRAC_PI_6 * r),
        )
    }

    fn to_grid_matrix(&self) -> Mat2 {
        self.to_world_matrix().inverse()
    }


    
    /// Translates a grid vector to a world-space position vector
    /// using this grid.
    pub fn to_world_position(&self, coordinate: GridVec) -> Vec3 {
        let to_world = self.to_world_matrix();
        let position = coordinate.axial().as_vec2();
        self.origin + (to_world * position).extend(0.0)
    }

    /// Translates a world position to a grid position
    ///
    /// Note that this takes a [`Vec2`] as opposed to a [`Vec3`]. The
    /// grid is only across two axes, so the z coordinate doesnt
    /// matter. Use [`Vec3::truncate`] if you want to get a grid
    /// position for a [`Vec3`].
    ///
    /// **WARNING*** Do not depend on the behavior described below,
    /// because of rounding errors in floating point arithmetic, a
    /// host of situations may arise that invalidate the following
    /// claims.
    ///
    /// Still, if you convert a point *exactly* on the border between
    /// two hexes (according to `to_grid_matrix()` in this source
    /// file), this function *should* select the hex on the inside of
    /// the local "collision rhombus". This means that of the nearest
    /// four hex centers to the point, the function will select one of
    /// the two that are closest to *each other*. If the point is on
    /// the edge between these two hexes, the function *should* prefer
    /// the one further to the right.
    pub fn to_grid_coordinate(&self, position: Vec2) -> GridVec {
        let position = position - self.origin.truncate();
        let grid_position = self.to_grid_matrix() * position;
        GridVec::hex_round(grid_position)
    }
}

/// The grid vector structure represents a grid tile in a hexagonal
/// coordinate system.
///
/// It's based on the cube-coordinate model found at
/// <https://www.redblobgames.com/grids/hexagons/>, so addition as
/// well as scalar multiplication "just work".
///
/// Note that the cube coordinates are *not the same* as truncated
/// world coordinates and you cannot simply convert world positions to
/// cube coordinates. You must have a [`Grid`] to convert between the
/// two. For right now, only converting from a grid coordinate to a
/// world coordinate is supported (through
/// [`Grid::to_world_position`]), but this is soon to change.
#[derive(Clone, Copy, Component, PartialEq, Eq, Hash, Default, Debug)]
pub struct GridVec {
    vec: IVec3,
}

/// This error is returned when [`GridVec::try_from`] is called on an
/// [`IVec3`] who's component-sum is not zero.
///
/// The component sum must be zero to specify cube coordinates because
/// the cube coordinates represent that planar subspace of 3D euclidean
/// space.
#[derive(Clone, Copy, Debug)]
pub struct CubeCoordinatesInvalid;

impl std::error::Error for CubeCoordinatesInvalid {}

impl std::fmt::Display for CubeCoordinatesInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "grid components did not sum to 0")
    }
}

impl TryFrom<IVec3> for GridVec {
    type Error = CubeCoordinatesInvalid;

    fn try_from(vec: IVec3) -> Result<GridVec, Self::Error> {
        let sum: i32 = vec.as_ref().iter().copied().sum();
        if sum != 0 {
            Err(CubeCoordinatesInvalid)
        } else {
            Ok(GridVec{ vec })
        }
    }
}

pub enum DimensionParseError {
    NoDirection,
    InvalidDirection(String),
    InvalidMagnitude(ParseIntError),
}

pub enum ArgumentParseError {
    TooFewArguments,
    MalformedDimension(DimensionParseError),
}

fn parse_dimension(dimension: &str) -> Result<GridVec, ArgumentParseError> {
    use ArgumentParseError as ArgError;
    use DimensionParseError as DimError;

    // Find the first character corresponding to a cardinal direction.
    let split_i = dimension.find(|c: char| {
        ['n','s','e','w'].contains(&c.to_ascii_lowercase())
    }).ok_or(ArgError::MalformedDimension(DimError::NoDirection))?;

    let (mag, dir) = dimension.split_at(split_i);

    let dir = match dir {
        "n" => GridVec::NORTH,
        "nw" => GridVec::NORTHWEST,
        "ne" => GridVec::NORTHEAST,
        "s" => GridVec::SOUTH,
        "sw" => GridVec::SOUTHWEST,
        "se" => GridVec::SOUTHEAST,
        _ => return Err(ArgError::MalformedDimension(
            DimError::InvalidDirection(String::from(dir))
        )),
    };

    let mag: i32 = mag.parse().map_err(|e| ArgError::MalformedDimension(DimError::InvalidMagnitude(e)))?;

    Ok(mag * dir)

}

impl TryFrom<Vec<&str>> for GridVec {
    type Error = ArgumentParseError;

    // Refereced the wikipedia page on EBNF
    // https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form
    // for a reminder on EBNF.
    /// Tries to parse a grid vector from the `args` (arguments)
    /// vector.
    ///
    /// There should be one or two arguments for each grid
    /// vector. Each argument `dim` should be a grid vector
    /// specification in the (EBNF) form below.
    ///
    /// ```ebnf
    /// dim = magnitude, direction
    /// magnitude = [-], digit, {digit}
    /// digit = '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'
    /// direction = 'n'|'nw'|'ne'|'s'|'sw'|'se'
    /// ```
    ///
    /// The n, nw, ne, s, sw, and se terminals correspond to the
    /// north, northwest, northeast, south, southwest, and southeast
    /// grid directions respectively. The magnitude must be
    /// representable as a 32 bit signed integer.
    fn try_from(mut args: Vec<&str>) -> Result<GridVec, Self::Error> {
        let mut iter = args.into_iter();
        let dimensions = [
            iter.next().ok_or(Self::Error::TooFewArguments)?,
            iter.next().unwrap_or("0n"),
        ];

        let mut grid_dims = Vec::new();

        for d in dimensions {
            grid_dims.push(parse_dimension(d)?);
        }

        Ok(grid_dims.into_iter().sum())
    }
}

impl GridVec {
    /// The unit grid vector pointing north
    ///
    /// The grid vector referring to the neighbor tile on the positive
    /// y world axis (when added to a given grid vector representing a
    /// tile).
    pub const NORTH: Self = GridVec { vec: IVec3 { x: 0, y: 1, z: -1} };

    /// The unit grid vector pointing south
    ///
    /// The grid vector referring to the neighbor tile on the negative
    /// y world axis (when added to a given grid vector representing a
    /// tile).
    pub const SOUTH: Self = GridVec { vec: IVec3 { x: 0, y: -1, z: 1} };

    /// The unit grid vector pointing roughly northeast
    ///
    /// The grid vector to giving the neighbor along the positive y and
    /// positive x axis (when added to a given
    /// grid vector representing a tile).
    pub const NORTHEAST: Self = GridVec { vec: IVec3 { x: 1, y: 0, z: -1} };

    /// The unit grid vector pointing roughly southeast
    ///
    /// The grid vector giving the neighbor in the direction of the
    /// positive y and negative x world axes (when added to a given
    /// grid vector representing a tile).
    pub const SOUTHEAST: Self = GridVec { vec: IVec3 { x: 1, y: -1, z: 0} };

    /// The unit grid vector pointing roughly northwest
    ///
    /// The grid vector giving the neighbor in the direction of the
    /// positive y and negative x world axes. (when added to a given
    /// grid vector representing a tile).
    pub const NORTHWEST: Self = GridVec { vec: IVec3 { x: -1, y: 1, z: 0} };

    /// The unit grid vector pointing roughly southwest
    ///
    /// The grid vector giving the neighbor in the direction of the
    /// negative y and negative x world axes. (when added to a given
    /// grid vector representing a tile).
    pub const SOUTHWEST: Self = GridVec { vec: IVec3 { x: -1, y: 0, z: 1} };

    /// The additive identity grid vector
    ///
    /// The zero grid vector is the additive identity vector for the
    /// GridVec vector space. That is to say for any GridVec V: V +
    /// ZERO = V.
    pub const ZERO: Self = GridVec { vec: IVec3::ZERO };

    /// A manual constructor taking the cube coordinates of the vector
    ///
    /// For an overview of cube coordinates, please see the helpful
    /// reference at <https://www.redblobgames.com/grids/hexagons/> We
    /// used that exact reference to implement this system, so it
    /// should give you a good idea of how the grid vectors work.
    pub fn new(cube_x: i32, cube_y: i32, cube_z: i32) -> Result<GridVec, CubeCoordinatesInvalid> {
        Self::try_from(IVec3::new(cube_x, cube_y, cube_z))
    }

    /// Returns the "axial" coordinates
    ///
    /// These are two of the cube coordinates, which are still
    /// unambiguous because of the constraint on cube coordinates.
    /// Again, check out the redblobgames reference for more.
    pub fn axial(&self) -> IVec2 {
        self.vec.truncate()
    }

    /// Extends the given axial grid coordinate into a cube coordinate
    pub fn from_axial(vec: IVec2) -> Self {
        Self { vec: vec.extend(-vec.x-vec.y), }
    }

    /// Takes a fractional axial coordinate and turns it into a cube
    /// coordinate.
    pub fn hex_round(vec: Vec2) -> Self {
        use bevy::math::Vec2Swizzles;
        let fractional = vec.fract();
        // Just trust me
        let fractional = fractional + fractional.yx() * SIN_FRAC_PI_6;

        let half = Vec2::splat(0.5);
        let one = Vec2::splat(1.0);
        
        let axial = if fractional.cmplt(half).all() {
            vec.floor()
        } else if fractional.cmpgt(one).all() {
            vec.ceil()
        } else if fractional.x < fractional.y {
            Vec2::new(vec.x.floor(), vec.y.ceil())
        } else {
            Vec2::new(vec.x.ceil(), vec.y.floor())
        };

        Self::from_axial(axial.as_ivec2())
    }

    /// Returns an array of all neighbors for `self`
    pub fn neighbors(self) -> [GridVec; 6] {
        [
            self + GridVec::SOUTH,
            self + GridVec::SOUTHEAST,
            self + GridVec::SOUTHWEST,
            self + GridVec::NORTH,
            self + GridVec::NORTHEAST,
            self + GridVec::NORTHWEST,
        ]
    }
}

impl std::ops::Add for GridVec {
    type Output = Self;
    fn add(self, rhs: GridVec) -> GridVec {
        GridVec::try_from(self.vec + rhs.vec).unwrap()
    }
}

impl std::iter::Sum for GridVec {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> GridVec {
        iter.fold(GridVec::ZERO, |sum, vec| sum + vec)
    }
}

impl std::ops::Sub for GridVec {
    type Output = Self;
    fn sub(self, rhs: GridVec) -> GridVec {
        GridVec::try_from(self.vec - rhs.vec).unwrap()
    }
}

impl std::ops::Mul<i32> for GridVec {
    type Output = Self;
    fn mul(self, rhs: i32) -> GridVec {
        GridVec::try_from(self.vec * rhs).unwrap()
    }
}

impl std::ops::Mul<GridVec> for i32 {
    type Output = GridVec;
    fn mul(self, rhs: GridVec) -> GridVec {
        GridVec::try_from(self * rhs.vec).unwrap()
    }
}

#[cfg(test)]
mod test {
    mod world_to_grid {
        const EPSILON: f32 = 0.0001;
        use super::super::*;

        fn grids() -> [Grid; 3] {
            [
                Grid {
                    major_radius: 30.0,
                    origin: Vec3::new(1.0, 2.0, 3.0),
                    tiles: HashMap::new(),
                },
                Grid::default(),
                Grid {
                    major_radius: 150.0,
                    origin: Vec3::new(200.0, 400.0, 8.0),
                    tiles: HashMap::new(),
                },
            ]
        }
        
        #[test]
        fn origin_north_border() {
            for grid in grids() {
                let inside_r = COS_FRAC_PI_6 * grid.major_radius - EPSILON;
                let outside_r = COS_FRAC_PI_6 * grid.major_radius + EPSILON;

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + Vec2::Y * inside_r);
                let expected = GridVec::ZERO;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + Vec2::Y * outside_r);
                let expected = GridVec::NORTH;
                assert_eq!(actual, expected);
            }
        }

                #[test]
        fn origin_south_border() {
            for grid in grids() {
                let inside_r = COS_FRAC_PI_6 * grid.major_radius - EPSILON;
                let outside_r = COS_FRAC_PI_6 * grid.major_radius + EPSILON;

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() - Vec2::Y * inside_r);
                let expected = GridVec::ZERO;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() - Vec2::Y * outside_r);
                let expected = GridVec::SOUTH;
                assert_eq!(actual, expected);
            }
        }

        #[test]
        fn origin_east_corner() {
            for grid in grids() {
                let inside_r = grid.major_radius - EPSILON;
                let outside_r = grid.major_radius + EPSILON;

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + Vec2::X * inside_r);
                let expected = GridVec::ZERO;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + Vec2::X * outside_r);
                let expected = [GridVec::NORTHEAST, GridVec::SOUTHEAST];
                assert!(expected.contains(&actual));
            }
        }

        #[test]
        fn origin_west_corner() {
            for grid in grids() {
                let inside_r = grid.major_radius - EPSILON;
                let outside_r = grid.major_radius + EPSILON;

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() - Vec2::X * inside_r);
                let expected = GridVec::ZERO;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() - Vec2::X * outside_r);
                let expected = [GridVec::NORTHWEST, GridVec::SOUTHWEST];
                assert!(expected.contains(&actual));
            }
        }

        #[test]
        fn origin_northeast_corner() {
            for grid in grids() {

                let vector = Vec2::new(grid.major_radius / 2.0, grid.major_radius * COS_FRAC_PI_6);
                let inside_vec = vector * (1.0 - EPSILON);
                let outside_vec = vector * (1.0 + EPSILON);

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + inside_vec);
                let expected = GridVec::ZERO;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + outside_vec);
                let expected = [GridVec::NORTH, GridVec::NORTHEAST];
                assert!(expected.contains(&actual));
            }
        }

        #[test]
        fn north_northeast_corner() {
            for grid in grids() {
                let vector = Vec2::new(
                    grid.major_radius / 2.0,
                    grid.major_radius * COS_FRAC_PI_6 * 3.0,
                );

                let inside_vec = vector * (1.0 - EPSILON);
                let outside_vec = vector * (1.0 + EPSILON);

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + inside_vec);
                let expected = GridVec::NORTH;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + outside_vec);
                let expected = [GridVec::NORTH * 2, GridVec::NORTH + GridVec::NORTHEAST];
                assert!(expected.contains(&actual));
            }
        }

        #[test]
        fn two_southwest_northwest_corner() {
            for grid in grids() {
                let vector = Vec2::new(
                    -grid.major_radius * 2.5,
                    -grid.major_radius * COS_FRAC_PI_6,
                );

                let inside_vec = vector - Vec2::new(0.0, EPSILON);
                let outside_vec = vector + Vec2::new(0.0, EPSILON);

                // Test inside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + inside_vec);
                let expected = GridVec::SOUTHWEST * 2;
                assert_eq!(actual, expected);

                // Test outside
                let actual = grid.to_grid_coordinate(grid.origin.truncate() + outside_vec);
                let expected = GridVec::SOUTHWEST * 2 + GridVec::NORTH;
                assert_eq!(actual, expected);
            }
        }
    }
}
