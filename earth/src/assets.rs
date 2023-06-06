use bevy::{
    asset::LoadState,
    prelude::*,
    render::{
        render_resource::AddressMode,
        texture::ImageSampler,
    },
};

use std::collections::VecDeque;

/// This plugin is responsible for setting up repeat sampling on
/// terrain textures
pub struct AssetPlugin;

/// This function loads all necessary textures for a given terrain material
///
/// The `name` argument is used to determine which textures to
/// load. The path loaded (through [`AssetServer::load`]) is
/// `textures/{name}/{name}_{suffix}.jpg`, where `{suffix}` is from
/// the following list.
///
/// - "albedo" (for the base color)
/// - "arm" (for roughness/metallic)
/// - "normal" (for the tangent-space OpenGL style normal map)
pub fn load_terrain_material<S: Into<String>>(
    name: S,
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    images_to_repeat: &mut RepeatSampleImageQueue,
) -> Handle<StandardMaterial> {
    let name: String = name.into();
    let albedo = asset_server.load(format!("textures/{name}/{name}_albedo.jpg"));
    let metallic_roughness = asset_server.load(format!("textures/{name}/{name}_arm.jpg"));
    let normal = asset_server.load(format!("textures/{name}/{name}_normal.jpg"));

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(albedo.clone()),
        metallic_roughness_texture: Some(metallic_roughness.clone()),
        normal_map_texture: Some(normal.clone()),
        ..default()
    });

    // The default sampler should be changed to repeat these on U and V axes.
    // We enqueue each of these into the repeating image queue.
    images_to_repeat.0.push_back((albedo.clone_weak(), material.clone_weak()));
    images_to_repeat.0.push_back((metallic_roughness.clone_weak(), material.clone_weak()));
    images_to_repeat.0.push_back((normal.clone_weak(), material.clone_weak()));

    material
}

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RepeatSampleImageQueue::default())
            .add_system(use_repeat_texture_samplers);
    }
}

/// This resource holds handles to the images that should be updated
/// to be sampled in a repeating "tiled" way, as well as the material
/// using them.
#[derive(Resource, Clone, Debug, Default)]
pub struct RepeatSampleImageQueue(VecDeque<(Handle<Image>, Handle<StandardMaterial>)>);

/// This system takes care of dequeueing the images enqueued by
/// load_terrain_materials.
fn use_repeat_texture_samplers(
    asset_server: Res<AssetServer>,
    mut images_to_repeat: ResMut<RepeatSampleImageQueue>,
    mut images: ResMut<Assets<Image>>,
    mut events: EventWriter<AssetEvent<StandardMaterial>>,
) {
    if let Some((handle, material_handle)) = images_to_repeat.0.front() {
        if asset_server.get_load_state(handle.clone()) == LoadState::Loaded {
            let image = images.get_mut(handle).unwrap();
            let mut sampler_descriptor = ImageSampler::linear_descriptor();
            sampler_descriptor.address_mode_u = AddressMode::Repeat;
            sampler_descriptor.address_mode_v = AddressMode::Repeat;
            image.sampler_descriptor = ImageSampler::Descriptor(sampler_descriptor);

            // Also update the dependent material
            events.send(AssetEvent::Modified{ handle: material_handle.clone_weak() });

            images_to_repeat.0.pop_front();
        }
    }
}
