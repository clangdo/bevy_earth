use bevy::prelude::*;

use super::{
    VertexData,
    error::SubdivisionError,
};

// Just over 1M vertices (not counting duplicates).
pub const MAX_MESH_RES: u32 = 0xA;

fn num_tris_ok(resolution: u32) -> Result<(), SubdivisionError> {
    if resolution > MAX_MESH_RES {
        Err(SubdivisionError::TooManySubdivisions {
            requested: resolution,
            limit: MAX_MESH_RES,
        })
    } else {
        Ok(())
    }
}

fn extend_with_quad(
    vertex_data: &mut VertexData,
    begin: Vec3,
    major: Vec3,
    minor: Vec3,
    uv_begin: Vec2,
    uv_size: Vec2,
) {
    let quad_corners = [
        begin,
        begin + major,
        begin + minor,
        begin + major + minor,
    ];

    let uv_corners = [
        uv_begin,
        uv_begin + uv_size * Vec2::Y,
        uv_begin + uv_size * Vec2::X,
        uv_begin + uv_size,
    ];

    // Iterate through the quad indices to make two triangles.
    for index in [0, 1, 2, 1, 3, 2] {
        vertex_data.positions.push(quad_corners[index].to_array());
        vertex_data.uvs.push(uv_corners[index].to_array());
    }
}

fn list_subdivided_vertex_data(subdivisions: u32, dimensions: f32) -> VertexData {
    let mut data = VertexData::new();
    let num_quad_rows = 2_u32.pow(subdivisions);
    let num_quad_columns = num_quad_rows;

    let step = dimensions / num_quad_rows as f32;
    let uv_step = 1.0 / num_quad_rows as f32;

    for row in 0..num_quad_rows {
        let y_coord = row as f32 * step - dimensions / 2.0;
        for col in 0..num_quad_columns {
            let x_coord = col as f32 * step - dimensions / 2.0;

            // The first vertex of the quad
            let begin = Vec3::new(x_coord, y_coord, 0.0);
            // The UV of the first vertex of the quad.
            let uv_begin = Vec2::new(row as f32 * uv_step, col as f32 * uv_step);

            extend_with_quad(
                &mut data,
                begin,
                step * Vec3::X,
                step * Vec3::Y,
                uv_begin,
                Vec2::splat(uv_step),
            );
        }
    }

    data
}

/// Create a new subdivided plane
///
/// This will return an error if subdivisions is higher than
/// [MAX_MESH_RES]. Which would create excessive vertex counts.
pub fn new(subdivisions: u32, dimensions: f32) -> Result<Mesh, SubdivisionError> {
    num_tris_ok(subdivisions)?;

    let vertex_data = list_subdivided_vertex_data(subdivisions, dimensions);

    Ok(vertex_data.into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolution_outside_boundary() {
        let result = super::new(MAX_MESH_RES + 1, 1.0);

        let expected_error = SubdivisionError::TooManySubdivisions {
            requested: MAX_MESH_RES + 1,
            limit: MAX_MESH_RES,
        };

        let resultant_error = result.unwrap_err();

        assert_eq!(expected_error, resultant_error)
    }

    #[test]
    fn resolution_inside_boundary() {
        let result = super::new(MAX_MESH_RES, 1.0);
        assert!(result.is_ok())
    }
}
