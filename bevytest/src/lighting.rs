mod ambient;
pub use ambient::Lighting as AmbientLighting;
pub use ambient::Settings as AmbientSettings;

mod day_night_cycle;
pub use day_night_cycle::Lighting as DayNightCycleLighting;
pub use day_night_cycle::Settings as DayNightCycleSettings;

use bevy::prelude::*;
    
/// These settings set up the lighting for a scene.
#[derive(Clone, Copy, Debug)]
pub enum Settings {
    Ambient(AmbientSettings),
    DayNightCycle(DayNightCycleSettings),
}

impl Default for Settings {
    fn default() -> Settings {
        Settings::Ambient(AmbientSettings::default())
    }
}

pub struct Lighting {
    settings: Settings,
}

impl Lighting {
    pub fn new(settings: Settings) -> Lighting {
        Lighting { settings }
    }
}

impl Plugin for Lighting {
    fn build(&self, app: &mut App) {
        match self.settings {
            Settings::Ambient(s) => app.add_plugin(AmbientLighting{ settings: s }),
            Settings::DayNightCycle(s) => app.add_plugin(DayNightCycleLighting{ settings: s }),
        };
    }
}
