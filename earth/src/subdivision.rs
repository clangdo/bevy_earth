use bevy::{
    prelude::*,
    render::mesh::{
        PrimitiveTopology,
        Indices,
    },
};

pub mod error;



pub mod hexagon;

#[allow(dead_code)]
pub mod triangle;
#[allow(dead_code)]
pub mod plane;

pub use triangle::new as new_triangle;

pub struct VertexData {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    indices: Option<Vec<u32>>,
}

#[allow(dead_code)]
impl VertexData {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            indices: None,
        }
    }

    fn new_indexed() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            indices: Some(Vec::new()),
        }
    }
}

impl From<VertexData> for Mesh {
    fn from(data: VertexData) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, data.tangents);
        mesh.set_indices(data.indices.map(Indices::U32));

        mesh
    }
}
