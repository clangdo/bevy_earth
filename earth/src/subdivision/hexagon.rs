use bevy::prelude::*;

use super::{
    error::SubdivisionError,
    VertexData,
    triangle,
};

const MAX_SUBDIVISIONS: u32 = 10;

pub fn new(subdivisions: u32, major_diameter: f32, uv_scale: f32) -> Result<Mesh, SubdivisionError> {
    if subdivisions > MAX_SUBDIVISIONS {
        return Err(SubdivisionError::TooManySubdivisions{
            requested: subdivisions,
            limit: MAX_SUBDIVISIONS,
        })
    }

    let triangle_face_rows = 2usize.pow(subdivisions);

    let mut data = VertexData::new_indexed();
    
    use std::f32::consts::{FRAC_PI_3, FRAC_PI_2};
    for triangle_index in 0..6 {
        let sector_angle = FRAC_PI_3 * triangle_index as f32;
        let axis = Vec2::from_angle(sector_angle + FRAC_PI_2);

        let build_info = triangle::TriangleBuildInfo {
            translation: Vec3::ZERO,
            axis,
            side_length: major_diameter / 2.0,
            rows: triangle_face_rows,
            uv_scale,
        };

        triangle::append_subdivided_vertex_data(&mut data, build_info);
    }

    Ok(data.into())
}
