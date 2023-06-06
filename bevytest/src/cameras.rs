use bevy::prelude::*;

mod azimuth_elevation;
mod two_dimensional;
// A utilities module for all camera rigs to use
mod camera_rig;

pub use azimuth_elevation::{
    CameraPlugin as AzimuthElevationCameraPlugin,
    Settings as AzimuthElevationSettings
};

pub use two_dimensional::{
    CameraPlugin as TwoDimensionalCameraPlugin,
    Settings as TwoDimensionalSettings,
};

/// These settings set up the scene camera itself
///
/// You can select one of the variants to select a type of camera. The
/// default camera is an
/// [`AzimuthElevation`](AzimuthElevationSettings) camera with default
/// settings.
#[derive(Clone, Debug)]
pub enum Settings {
    AzimuthElevation(AzimuthElevationSettings),
    TwoDimensional(TwoDimensionalSettings),
}

impl Default for Settings {
    fn default() -> Settings {
        Settings::AzimuthElevation(AzimuthElevationSettings::default())
    }
}

pub struct Cameras {
    pub settings: Settings,
}

impl Cameras {
    pub fn new(settings: Settings) -> Cameras {
        Cameras { settings }
    }
}

impl Plugin for Cameras {
    fn build(&self, app: &mut App) {
        match &self.settings {
            Settings::AzimuthElevation(settings) => {
                app.add_plugin(AzimuthElevationCameraPlugin { settings: settings.clone() });
            },
            Settings::TwoDimensional(_) => {
                app.add_plugin(TwoDimensionalCameraPlugin::default());
            }
        }
    }
}
