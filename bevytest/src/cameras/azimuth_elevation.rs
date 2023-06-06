// This started as a copy paste from
// https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

use bevy::{
    prelude::*,
    input::mouse::{MouseMotion, MouseWheel},
    window::PrimaryWindow,
};
use std::{f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU}, {ops::Add, ops::Mul}};
use super::camera_rig::*;

#[derive(Clone, Debug, Resource)]
pub struct Settings {
    /// The initial focus of the camera, defaults to the origin
    pub focus: Vec3,
    /// The initial distance the camera is from its focus, defaults to
    /// 10.0
    pub radius: f32,
    /// The basis matrix of the camera's coordinate system, defaults
    /// to Z up, X right, Y forward
    pub basis: Mat3,
    /// The initial azimuth of the camera in radians, defaults to 0.0
    pub azimuth: f32,
    /// The initial elevation of the camera in radians, defaults to
    /// 0.0
    pub elevation: f32,
    /// The minimum zoom of the camera, defaults to 0.5, This must be
    /// greater than 0 to avoid zoom locking
    pub min_zoom: f32,
    /// The default mouse button used to orbit, defaults to
    /// `MouseButton::Left`.
    pub orbit_button: MouseButton,
    /// The default mouse button used to pan, defaults to
    /// `MouseButton::Middle`.
    pub pan_button: MouseButton,
    /// The default key used to center new cameras, defaults to
    /// `KeyCode::C`.
    pub center_button: KeyCode,
    /// The fog settings for this camera
    pub fog_settings: FogSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 10.0,
            basis: UpDirection::Z.into(),
            azimuth: 0.0,
            elevation: 0.0,
            min_zoom: 0.5,
            orbit_button: MouseButton::Left,
            pan_button: MouseButton::Middle,
            center_button: KeyCode::C,
            fog_settings: FogSettings {
                falloff: FogFalloff::Exponential { density: 0.001 },
                ..default()
            },
        }
    }
}

impl Settings {
    /// Yeilds an azimuth elevation camera looking down with default ambient lighting.
    pub fn high_angle() -> Self {
        Self {
            elevation: FRAC_PI_4,
            ..default()
        }
    }
}

/// This plugin sets up a single azimuth-elevation camera according to `settings`.
pub struct CameraPlugin {
    pub settings: Settings,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .add_startup_system(spawn_rigged_camera)
            .add_system(reset_focus)
            .add_system(update_transform);
    }
}

/// This enum specifies and axial up direction to help getting a quick basis matrix.
///
/// You have to convert this type [`into`](std::convert::Into)
/// [`Mat3`] to use it. If you'd rather specify the basis matrix
/// manually you don't need to use this struct.
#[derive(Clone, Copy, Debug)]
pub enum UpDirection {
    X,
    Y,
    Z,
}

impl From<UpDirection> for Mat3 {
    fn from(up: UpDirection) -> Mat3 {
        match up {
            UpDirection::X => Mat3::from_cols(Vec3::Z, Vec3::Y, Vec3::X),
            UpDirection::Y => Mat3::from_cols(Vec3::X, -Vec3::Z, Vec3::Y),
            UpDirection::Z => Mat3::IDENTITY,
        }
    }
}

/// An azimuth-elevation orbit camera gimbal
///
/// When this is attached to a camera bundle it will allow the camera
/// to orbit around a focus with an azimuth and elevation. It also allows panning and zoom (physical, non-fov zoom).
#[derive(Component, Clone, Copy, Debug)]
struct Gimbal {
    initial_focus: Vec3,
    focus: Vec3,
    radius: f32,
    basis: Mat3,
    azimuth: f32,
    elevation: f32,
    min_zoom: f32,
    orbit_button: MouseButton,
    pan_button: MouseButton,
    center_button: KeyCode,
}

impl From<Settings> for Gimbal {
    fn from(settings: Settings) -> Gimbal {
        Gimbal {
            initial_focus: settings.focus,
            focus: settings.focus,
            radius: settings.radius,
            basis: settings.basis,
            azimuth: settings.azimuth,
            elevation: settings.elevation,
            min_zoom: settings.min_zoom,
            orbit_button: settings.orbit_button,
            pan_button: settings.pan_button,
            center_button: settings.center_button,
        }
    }
}

#[derive(Bundle)]
struct RiggedCameraBundle {
    camera: Camera3dBundle,
    fog_settings: FogSettings,
    rig: Gimbal,
}

impl From<Settings> for RiggedCameraBundle {
    fn from(settings: Settings) -> RiggedCameraBundle {
        let rig = Gimbal::from(settings.clone()); 
        let transform = Transform::from(rig);

        RiggedCameraBundle {
            camera: Camera3dBundle {
                transform,
                ..default()
            },
            fog_settings: settings.fog_settings,
            rig,
        }
    }
}

fn spawn_rigged_camera(mut commands: Commands, settings: Res<Settings>) {
    commands.spawn(RiggedCameraBundle::from(settings.clone()))
        .insert(Name::new("Azimuth Elevation Camera"));
}

fn reset_focus(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Gimbal>,
) {
    for mut gimbal in &mut query {
        if keyboard.pressed(gimbal.center_button) {
            gimbal.focus = gimbal.initial_focus;
        }
    }
}

fn update_transform(
    mouse: Res<Input<MouseButton>>,
    motion_events: EventReader<MouseMotion>,
    scroll_events: EventReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&mut Gimbal, &mut Transform, &Projection), With<Camera>>,
) {
    let mouse_motion: Vec2 = mouse_motion(motion_events);
    let scrolling: f32 = scrolling(scroll_events);

    let moved_mouse = mouse_motion.length_squared() > 0.0;
    let zooming = scrolling.abs() > 0.0;
    let primary_window = window.get_single()
        .expect("could not get primary window for az-el camera rig");
    let target_dimensions = window_dimensions(primary_window);

    // update cameras
    for (mut gimbal, mut transform, projection) in cameras.iter_mut() {
        let orbiting = moved_mouse & mouse.pressed(gimbal.orbit_button);
        let panning = moved_mouse & mouse.pressed(gimbal.pan_button);

        // Prioritize orbit over pan, don't do both at once
        if orbiting {
            gimbal.orbit(target_dimensions, mouse_motion);
        } else if panning {
            gimbal.pan(&transform, target_dimensions, mouse_motion, projection);
        }

        if zooming {
            gimbal.zoom(scrolling);
        }

        if gimbal.is_changed() {
            transform.clone_from(&Transform::from(gimbal.as_ref()));
        }
    }
}

impl Gimbal {
    fn orbit(&mut self, window_dimensions: Vec2, mouse_motion: Vec2) {
        let movement_scales = Vec2 { x: TAU, y: PI };
    
        let delta = mouse_motion / window_dimensions * movement_scales;

        self.azimuth -= delta.x;
        self.elevation += delta.y;

        self.elevation = self.elevation.clamp(-FRAC_PI_2, FRAC_PI_2);
    }

    fn pan(
        &mut self,
        transform: &Transform,
        target_dimensions: Vec2,
        mouse_motion: Vec2,
        projection: &Projection
    ) {
        // make panning distance independent of resolution and FOV,
        let panning = ar_fov_normalize(mouse_motion, target_dimensions, projection);

        // translate by local axes
        let mat = Mat3::from_quat(transform.rotation);
        let left = -mat.x_axis * panning.x;
        let up = mat.y_axis * panning.y;

        // make panning proportional to distance away from focus point
        let translation = (left + up) * self.radius;
        self.focus += translation;
    }

    /// Zooms the camera by [scrolling], keeping zoom > [self.min_zoom].
    fn zoom(&mut self, scrolling: f32) {
        self.radius -= scrolling * self.radius * 0.2;
        // dont allow zoom to reach zero or you get stuck
        self.radius = self.radius.max(self.min_zoom);
    }

    /// Calculates the rotation of the camera attached to this gimbal.
    fn get_rotation(&self) -> Quat {
        let yaw = Quat::from_axis_angle(self.basis.z_axis, self.azimuth);
        let pitch = Quat::from_axis_angle(self.basis.x_axis, FRAC_PI_2 - self.elevation);
        let roll = Quat::from_rotation_arc(Vec3::Z, self.basis.z_axis);
        yaw * pitch * roll
    }

    /// This gets the translation of the camera, it takes one argument
    /// `rotation`, which allows for a small optimization if one
    /// already has the camera's rotation calculated. If provided as
    /// `None` then the rotation will be calculated with
    /// `get_rotation`.
    fn get_translation(&self, rotation: Option<Quat>) -> Vec3 {
        rotation.unwrap_or_else(|| self.get_rotation())
            .mul(Vec3::new(0.0, 0.0, self.radius))
            .add(self.focus)
    }
}

impl From<Gimbal> for Transform {
    fn from(gimbal: Gimbal) -> Transform {
        From::<&Gimbal>::from(&gimbal)
    }
}

impl From<&mut Gimbal> for Transform {
    fn from(gimbal: &mut Gimbal) -> Transform {
        From::<&Gimbal>::from(gimbal)
    }
}

impl From<&Gimbal> for Transform {
    fn from(gimbal: &Gimbal) -> Transform {
        let rotation = gimbal.get_rotation();
        Transform {
            translation: gimbal.get_translation(Some(rotation)),
            rotation,
            ..default()
        }
    }
}
