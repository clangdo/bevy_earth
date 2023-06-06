/* I came up with this recursive algorithm, it starts at one vertex of
 * the triangle and "splits" tree-wise to either side of an "axis"
 * vector. The right "child" vertex won't split to the left to avoid
 * collisions.
 *
 * There are probably numerous other (better) ways to do this, but
 * this should have \\(O(n)\\) complexity, where \\(n\\) is the number
 * of vertices, which isn't too bad.
 *
 * The arrow shows the direction of the `axis` vector.
 *
 * ```
 *        * --------     ↓ axis
 *       / \       |           
 *      *   *      |            
 *     /   / \     | rows = 3 (in this case)
 *    *   *   *    |    
 *   /   /   / \   |
 *  *   *   *   * --
 * ```
 * 
 * Note that this creates equilateral triangles, and the size of each
 * triangle depends on the `side_length` requested (which is of the
 * large triangle).
 */

use bevy::prelude::*;

use super::{
    VertexData,
    error::SubdivisionError,
};

const MAX_SUBDIVISIONS: u32 = 10;
const UV_SCALE: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
struct Edges {
    left: Option<Vec3>,
    right: Vec3,
}

fn split_from<F>(
    data: &mut VertexData,
    vertex: Vec3,
    edges_out: Edges,
    rows: usize,
    convert_to_uv: &F,
) where
    F: Fn(Vec3) -> Vec2
{
    data.positions.push(vertex.into());
    let uv = convert_to_uv(vertex);
    data.uvs.push(uv.into());
    // TODO: Make these respect up...
    data.normals.push(Vec3::Z.into());
    data.tangents.push(Vec3::X.extend(0.0).into());

    if rows == 0 {
        return;
    }
    
    if let Edges { left: Some(left), right } = edges_out {
        let right_edges_out = Edges{ left: None, right };

        split_from(data, vertex + right, right_edges_out, rows - 1, convert_to_uv);
        split_from(data, vertex + left, edges_out, rows - 1, convert_to_uv);
    } else {
        split_from(data, vertex + edges_out.right, edges_out, rows - 1, convert_to_uv);
    }
}

pub struct TriangleBuildInfo {
    pub translation: Vec3,
    pub axis: Vec2,
    pub side_length: f32,
    pub rows: usize,
    pub uv_scale: f32,
}

fn list_subdivided_vertex_data(
    settings: TriangleBuildInfo,
) -> VertexData {
    let mut data = VertexData::new_indexed();
    append_subdivided_vertex_data(&mut data, settings);

    data
}

pub fn append_subdivided_vertex_data(
    data: &mut VertexData,
    build_info: TriangleBuildInfo,
) {

    let angle = std::f32::consts::FRAC_PI_6;
    let begin = data.positions.len();
    
    let rotations = (
        Quat::from_axis_angle(Vec3::Z, angle),
        Quat::from_axis_angle(-Vec3::Z, angle),
    );

    let edge_length = build_info.side_length / build_info.rows as f32;
    let axial_edge = build_info.axis.extend(0.0).normalize() * edge_length;

    let edges = Edges {
        left: Some(rotations.0 * axial_edge),
        right: rotations.1 * axial_edge,
    };

    split_from(
        data,
        build_info.translation,
        edges,
        build_info.rows,
        &|pos: Vec3| {
            let scale = build_info.uv_scale;
            0.5 * scale * (pos.truncate() + build_info.side_length)
        },                
    );

    fill_indices(data, begin, build_info.rows);
}

fn fill_indices(data: &mut VertexData, begin: usize, rows: usize) {
    let num_vertices = (rows + 1) * (rows + 2) / 2;
    let mut row_local_index = 0;
    let mut row_size = rows + 1;

    let vertex_range = (begin as i64)..(begin as i64 + num_vertices as i64);

    for vertex_index in vertex_range.clone() {
        if row_local_index == row_size - 1 {
            row_local_index = 0;
            row_size -= 1;
            continue;
        }

        let triangles = vec!{
            vec!{
                vertex_index,
                vertex_index + 1,
                vertex_index + row_size as i64,
            },
            vec!{
                vertex_index,
                vertex_index - row_size as i64,
                vertex_index + 1,
            },
        };

        let mut filtered_indices: Vec<u32> = triangles
            .into_iter()
            .filter(|triangle| triangle.iter().all(
                |vertex| vertex_range.contains(vertex)
            ))
            .flatten()
            .map(|index| index as u32)
            .collect();

        data.indices.get_or_insert(Vec::new()).append(&mut filtered_indices);

        row_local_index += 1;
    }
}

/// Creates a subdivided (tesselated) equilateral triangle mesh.
///
/// It starts at "vertex" and creates rows of triangles. The number of
/// rows created is equal to `2.pow(subdivisions)`. It
/// creates these rows in the opposite of `vertex_direction`, that is
/// to say it makes the given `vertex` "point" in the direction of
/// `vertex_direction`.
///
/// When complete, each side of the resulting mesh will be of length
/// `side_length` and each edge will be of length `side_length / rows`
///
/// For now, this always creates the triangles such that Z is up,
/// therefore `vertex_direction` represents the orientation of the
/// resulting triangle on the xy-plane.
///
/// Do not rely on the triangle list (or mesh indices) being in a
/// certain arrangement without carefully studying the logic in the
/// private functions of this file.
pub fn new(
    vertex: Vec3,
    subdivisions: u32,
    side_length: f32,
    vertex_direction: Vec2,
    uv_scale: f32,
) -> Result<Mesh, SubdivisionError> {
    if subdivisions > MAX_SUBDIVISIONS {
        return Err(SubdivisionError::TooManySubdivisions{
            requested: subdivisions,
            limit: MAX_SUBDIVISIONS
        });
    }

    let rows = 2usize.pow(subdivisions);
    let build_info = TriangleBuildInfo {
        translation: vertex,
        axis: -vertex_direction,
        side_length,
        rows,
        uv_scale,
    };

    let vertex_data = list_subdivided_vertex_data(build_info);
    Ok(vertex_data.into())
}
