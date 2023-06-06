use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Lod {
    pub min_distance: f32,
    pub scene: Handle<Scene>,
}

#[derive(Component, Clone, Debug)]
pub struct LodInfo {
    pub lod0: Handle<Scene>,
    pub lods: Vec<Lod>,
    pub cull_distance: f32,
}

#[derive(Bundle)]
pub struct LodSceneBundle {
    pub lod_info: LodInfo,
    pub scene_bundle: SceneBundle,
}

pub struct LodPlugin;

impl Plugin for LodPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_lods);
    }
}

fn update_lods(
    mut commands: Commands,
    cameras: Query<(&Camera, Ref<GlobalTransform>)>,
    mut lodded_scenes: Query<(
        Entity,
        &mut Handle<Scene>,
        &GlobalTransform,
        &LodInfo,
        &mut Visibility,
    )>,
) {
    // Find the first active camera with a transform
    let camera = cameras.into_iter().find(|(camera, _)| camera.is_active);
    
    // If there's no active camera there's nothing to do
    if camera.is_none() {
        return;
    }

    // Otherwise unwrap it and it's transform
    let (_, camera_transform) = camera.unwrap();

    // If it hasn't moved we don't have to do anything
    if !camera_transform.is_changed() {
        return;
    }

    // Iterate through LodScenes and update their level of detail or
    // cull them if they're far enough away
    for (entity, mut scene, scene_transform, lod_info, mut visibility) in lodded_scenes.iter_mut() {
        use std::ops::Sub;
        let scene_position = scene_transform.translation();
        let camera_position = camera_transform.translation();
        let camera_distance = scene_position.sub(camera_position).length();

        if lod_info.cull_distance < camera_distance {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Inherited;
        }

        let scene_lod = lod_info
            .lods
            .iter()
            .find(|lod| lod.min_distance < camera_distance)
            .map(|lod| lod.scene.clone_weak())
            .unwrap_or(lod_info.lod0.clone_weak());

        let scene_different = scene_lod.id() != scene.id();
        if  scene_different {
            *scene = scene_lod;
            commands.entity(entity).clear_children();
        }
    }
}
