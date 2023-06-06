use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Resource)]
pub struct Settings {
    /// The color of the light, defaults to `Color::WHITE`
    pub color: Color,
    /// The multiplier, defaults to 1.0
    pub brightness: f32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            color: Color::WHITE,
            brightness: 1.0,
        }
    }
}

pub struct Lighting {
    pub settings: Settings
}

impl Plugin for Lighting {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings)
            .add_startup_system(add_light);
    }
}

impl From<Settings> for AmbientLight {
    fn from(settings: Settings) -> AmbientLight {
        AmbientLight {
            color: settings.color,
            brightness: settings.brightness,
        }
    }
}

fn add_light(mut commands: Commands, settings: Res<Settings>) {
    commands.insert_resource(AmbientLight::from(*settings));
}
