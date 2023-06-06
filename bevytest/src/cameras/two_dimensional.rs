use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use super::camera_rig::*;

#[derive(Clone, Copy, Debug, Resource)]
pub struct Settings {
    /// The button used to drag the camera around, defaults to `MouseButton::Left`
    pan_button: MouseButton,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            pan_button: MouseButton::Left,
        }
    }
}

/// This plugin implements a pannable "top-down" 2D camera.
#[derive(Debug, Default)]
pub struct CameraPlugin {
    settings: Settings,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings)
            .add_startup_system(spawn)
            .add_system(update);
    }
}

#[derive(Clone, Copy, Debug, Component)]
struct Slider {
    pan_button: MouseButton,
}

impl From<Settings> for Slider {
    fn from(settings: Settings) -> Self {
        Self {
            pan_button: settings.pan_button,
        }
    }
}


fn spawn(mut commands: Commands, settings: Res<Settings>) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(Slider::from(*settings))
        .insert(Name::new("2D Camera"));
}

fn update(
    mouse: Res<Input<MouseButton>>,
    motion_events: EventReader<MouseMotion>,
    scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&Slider, &mut OrthographicProjection, &mut Transform)>,
) {
    let mouse_motion = mouse_motion(motion_events);
    let scrolling = scrolling(scroll_events);

    let mouse_moved = mouse_motion.length_squared() > 0.0;
    let scrolled = scrolling.abs() > 0.0;

    for (slider, mut projection, mut transform) in &mut query {
        let panning = mouse_moved & mouse.pressed(slider.pan_button);
        let zooming = scrolled;

        if panning {
            transform.translation += projection.scale * mouse_motion.extend(0.0) * Vec3::new(-1.0, 1.0, 1.0);
        }

        if zooming {
            projection.scale *= 0.5_f32.powf(scrolling);
        }
    }
}
