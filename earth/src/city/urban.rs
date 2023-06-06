use bevy::{
    prelude::*
};

// holds the grass texture
#[derive(Clone, Debug, Resource)]
pub struct GroundTexture(pub Handle<Image>);
// holds the grass texture
#[derive(Clone, Debug, Resource)]
pub struct SideTexture(pub Handle<Image>);

// holds the name and scale of each natural asset
pub const ASSETS: [CityObject; 2] = [
    CityObject {
        name: "pine",
        scale: 1.4, // was .4
    },
    CityObject {
        name: "floor",
        scale: 13.0, // was .4
    },
];

// struct for the natural objects, given the clone trait
#[derive(Clone, Default, Resource)]
pub struct CityObject {
    pub name: &'static str,
    pub scale: f32,
}


// creates settings for the texture
// returns a standar mat (get handle)
pub fn create_material(texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(texture),
        cull_mode: None,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        metallic: 0.0,
        ..default()
    }
}

// creates a quad (for the ground base)
// modify so it takes meshes + w,l
// add mesh (would give handle for mesh) (add the result of create quad to mesh assets)
pub fn create_quad(width: f32, length: f32) -> Mesh {
    Mesh::from(shape::Quad::new(Vec2::new(
        width,
        length,
    )))
}

pub fn generate_hexagon_points(size: f32) -> Vec<[f32; 3]> {
    let angle_degrees = 60.0;
    let mut points = Vec::new();

    for i in 0..6 {
        let angle_rad = (angle_degrees * i as f32).to_radians();
        let x = size * angle_rad.cos();
        let y = size * angle_rad.sin();
        points.push([x, y, 0.0]);
    }

    points
}

pub fn decide_coords(x: i32, _y: i32, z: i32) -> (f32, f32) {
    let col = z as f32;
    let row = (x as f32) + ((z as f32) / 2.0); 
    // + (((z as f32)+(y as f32))/2.0);
    (col*15.000, row*69.282/4.0)
}
