use bevy::prelude::*;

use fastrand;

use crate::lod::*;

mod forest;

const MAX_SPAWN_ATTEMPTS: usize = 100;
const SPAWN_RADIUS: f32 = 40.0;

/// The plugin that loads all assets for the natural environment
pub struct NaturePlugin;

/// This structure represents a single gltf asset for the natural
/// environment.
///
/// This structure explains not only the asset's name, but also its
/// exclusion radius, spawn density, and other rendering properties.
#[derive(Clone, Default, Resource, Component)]
pub struct NaturalObject {
    
    /// This is the name of the asset in `snake_case`. Note that this
    /// will also be used to create a label for the asset (but it will
    /// be reformatted into Title Case in the label).
    pub name: &'static str,

    /// This is the radius marking this assets footprint; it's used
    /// in creating collision cylinders
    pub radius: f32,
    
    /// This is the count of this asset that attempts to spawn in each
    /// forest tile.
    ///
    /// The amount actually spawned depends on whether there is space
    /// for all of the objects that attempt to spawn.
    pub count: usize,
    
    /// This is the distance from the camera at which this object is
    /// no longer rendered.
    pub cull_distance: f32,

    /// The number of levels of detail in addition to the maximum
    /// level
    ///
    /// These should be saved alongside the original asset with "_l1,
    /// _l2, _l3..." suffixes to be automatically configured during
    /// loading.
    pub extra_lods: usize,

    /// How far one must be from the asset, past the previous lod
    /// cutoff, to reduce to the next level of detail
    pub lod_distance_step: f32,
}

pub use forest::AddForest;

#[derive(Clone, Copy, Component)]
struct SpawnCollider {
    center: Vec2,
    radius: f32,
}

impl SpawnCollider {
    fn is_colliding_with_any<'a, 'b: 'a, I: IntoIterator<Item = &'b Self>>(&'a self, others: I) -> bool {
        others.into_iter().any(|other_collider| self.is_colliding_with(other_collider))
    }

    fn is_colliding_with<'a, 'b: 'a>(&'a self, other: &'b Self) -> bool {
        let to_other = other.center - self.center;
        let min_distance_squared = (self.radius + other.radius).powi(2);
        to_other.length_squared() < min_distance_squared
    }
}

// Load textures and models when the nature plugin is loaded
impl Plugin for NaturePlugin {
    fn build(&self, app: &mut App){
        let forest_startup_systems = (forest::load_ground_material, forest::load_models);
        app.add_startup_systems(forest_startup_systems.in_base_set(StartupSet::PreStartup));
    }
}

fn create_spawn_tasks<'a, 'b, I>(
    asset_server: &'a AssetServer,
    assets: I,
) -> Vec<SpawnTask> where
    I: Iterator<Item = &'b NaturalObject>
{
    let mut spawn_tasks = Vec::new();

    for asset_info in assets {
        let scene_path = format!("models/{}.glb#Scene0", asset_info.name);
        let scene_handle = asset_server.get_handle(scene_path).clone();
        let mut lods = Vec::new();

        for lod_index in 1..=asset_info.extra_lods {
            let path = format!("models/{}_l{}.glb#Scene0", asset_info.name, lod_index);
            lods.push(Lod {
                scene: asset_server.get_handle(path).clone(),
                min_distance: asset_info.lod_distance_step * lod_index as f32,
            });
        }

        spawn_tasks.push(SpawnTask{
            scene: scene_handle,
            lods,
            properties: asset_info.clone(),
        });
    }

    spawn_tasks
}

struct SpawnTask {
    pub scene: Handle<Scene>,
    pub lods: Vec<Lod>,
    pub properties: NaturalObject,
}

impl SpawnTask {
    fn attempt(
        self,
        colliders: &mut Vec<SpawnCollider>,
        rng: &fastrand::Rng,
        builder: &mut WorldChildBuilder<'_>,
    ) {
        for _ in 0..self.properties.count {
            self.attempt_spawn_single(colliders, rng, builder)
        }
    }

    fn attempt_spawn_single(
        &self,
        colliders: &mut Vec<SpawnCollider>,
        rng: &fastrand::Rng,
        builder: &mut WorldChildBuilder<'_>,
    ) {
        for _ in 0..MAX_SPAWN_ATTEMPTS {
            use std::f32::consts::TAU;
            let random_radius = SPAWN_RADIUS * rng.f32();
            let random_angle = Vec2::from_angle(rng.f32() * TAU).extend(0.0);
            let random_position = random_radius * random_angle;
            let random_scale = rng.f32() * 0.5 + 0.5; // [0.5, 1.0)

            let collider = SpawnCollider {
                center: random_position.truncate(),
                radius: self.properties.radius * random_scale,
            };

            if !collider.is_colliding_with_any(colliders.iter()) {
                self.spawn_single(random_position, random_scale, builder);
                colliders.push(collider);
                // We've succeeded in placing an asset
                return;
            }
        }
    }
    
    fn spawn_single(&self, position: Vec3, scale: f32, builder: &mut WorldChildBuilder<'_>) {
        use std::f32::consts::{FRAC_PI_2, TAU};

        // Z is up, not the gltf standard Y
        let up_adjust_rotation = Quat::from_rotation_x(FRAC_PI_2);
        let random_rotation = Quat::from_rotation_z(fastrand::f32() * TAU);

        let transform = Transform::from_translation(position)
            .with_scale(Vec3::splat(scale))
            .with_rotation(random_rotation * up_adjust_rotation);

        builder.spawn(LodSceneBundle {
            lod_info: LodInfo {
                lod0: self.scene.clone(),
                lods: self.lods.clone(),
                cull_distance: self.properties.cull_distance,
            },
            scene_bundle: SceneBundle {
                scene: self.scene.clone_weak(),
                transform,
                ..default()
            },
        }).insert(Name::new(forest::to_title_case(self.properties.name)));
    }
}
