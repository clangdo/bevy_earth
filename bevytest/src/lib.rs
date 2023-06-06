//! This crate collects a number of useful boilerplate test
//! functions into a single place for easy access.
//!
//! Note that all of the settings and plugins implement
//! [`Default`](core::default::Default). Specifically, the default
//! [`ScenePlugin`] will use a centered, coordinate-system conformant,
//! azimuth-elevation camera with `LMB` to orbit, `MMB` to pan, and `c` to
//! recenter.
//!
//! The [`PerformanceMonitorPlugin`] shows simple frame times and
//! rates on the screen. Hopefully graphs of these quantities will
//! also be possible in future, though this is currently unimplemented.
//!
//! ```rust
//! // This is how you might setup a debug scene for testing
//! // Remember to set font path appropriately!
//! # let font_path  = "fonts/SourceCodePro-Regular.otf";
//! use bevy::prelude::*;
//! use bevytest::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(ScenePlugin::default())
//!     .add_plugin(PerformanceMonitorPlugin::with_font(font_path))
//!     // ...additional setup
//! #   ;
//! ```

use bevy::prelude::*;

mod cameras;
mod lighting;

pub use lighting::Settings as LightSettings;
pub use cameras::Settings as CameraSettings;

use cameras::Cameras;
use lighting::Lighting;

mod performance;
use performance::FrameTimePlugin;

/// This module allows you easy access to all the settings and types for simple scene setup.
///
/// Use it in the same way as the bevy prelude (or any other prelude).
/// ```
/// use bevytest::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        cameras::{
            AzimuthElevationSettings,
            TwoDimensionalSettings,
        },
        lighting::{
            DayNightCycleSettings,
            AmbientSettings,
        },
        PerformanceMonitorPlugin,
        ScenePlugin,
        SceneSettings,
        LightSettings,
        CameraSettings,
    };
}

/// These settings set up the [`DebugPlugin`].
///
/// Behind the scenes, these simply consist of a font path note this
/// is the same as any other [`bevy::asset::AssetPath`] passed to
/// [`AssetServer::load`].
#[derive(Clone, Debug, Resource)]
struct PerformanceMonitorSettings {
    font_path: String,
}

/// The debug plugin shows frames per second (fps) and frame render
/// times.
pub struct PerformanceMonitorPlugin {
    settings: PerformanceMonitorSettings
}

impl PerformanceMonitorPlugin {
    pub fn with_font<P>(font_path: P) -> Self where P: Into<String> {
        let settings = PerformanceMonitorSettings {
            font_path: font_path.into(),
        };

        Self {
            settings,
        }
    }
}

impl Plugin for PerformanceMonitorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .add_plugin(FrameTimePlugin);
        // TODO: Add a tick time plugin and maybe a console plugin.
    }
}

/// The scene settings are used to set up the scene plugin.
///
/// By default the scene will set up an azimuth-elevation camera 10
/// units away from the center of the scene with "c" as the center
/// key, left mouse button to orbit, and middle mouse button to pan.
///
/// The lighting will be ambient by default with intensity of `1.0` and a color of [`Color::WHITE`].
#[derive(Default)]
pub struct SceneSettings {
    pub cameras: CameraSettings,
    pub lighting: LightSettings,
}

/// The scene plugin helps set up a simple scene with Z up.  This is
/// the convention we're using, so it's great for testing.
#[derive(Default)]
pub struct ScenePlugin {
    settings: SceneSettings,
}

impl ScenePlugin {
    /// Yeilds a new scene plugin with the provided settings.
    pub fn with_settings(settings: SceneSettings) -> ScenePlugin {
        ScenePlugin {
            settings,
        }
    }
}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Cameras::new(self.settings.cameras.clone()))
            .add_plugin(Lighting::new(self.settings.lighting));
    }
}
