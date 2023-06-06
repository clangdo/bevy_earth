
// Test Grid Sizes 3
//
//

use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy::{
    prelude::*,
};
use bevytest::prelude::*;
use earth::{
    prelude::*,
    grid::hex::*,
};

fn main() {
    // Define the settings for the scene
    let settings = SceneSettings {
        cameras: CameraSettings::AzimuthElevation(
            AzimuthElevationSettings::high_angle()
        ),
        lighting: LightSettings::DayNightCycle(
            DayNightCycleSettings::default()
        ),
    };

    // Create a new Bevy application
    App::new()
        .add_plugins(DefaultPlugins) // Add default plugins
        .add_plugin(ScenePlugin::with_settings(settings)) // Add the scene plugin with the specified settings
        .add_plugin(PerformanceMonitorPlugin::with_font("fonts/source_code_pro/SourceCodePro-Regular.otf")) // Add a performance monitor plugin with a specific font
        .add_plugin(WorldInspectorPlugin::new()) // Add a world inspector plugin
        .add_plugin(grid::hex::GridPlugin { major_radius: 50000.0, origin: Vec3::new(1.0, 2.0, 3.0) }) // Add custom Earth plugins
        .add_plugin(city::CityPlugin)
        .add_startup_system(test_sizes) // Run the proc_gen system after the add_grid system
        .run(); // Run the application
}


fn test_sizes(mut commands: Commands) {

    // let zero_position: GridVec = GridVec::ZERO; // Initial position is (0, 0)
    let position: GridVec = GridVec::ZERO; // Current position

        // Add city entities around the current position
        commands.add(city::AddCity {
            grid_position: position,
            layout: 0,
        });
        commands.add(city::AddCity {
            grid_position: position + GridVec::SOUTH,
            layout: 0,
        });
        commands.add(city::AddCity {
            grid_position: position + GridVec::NORTH,
            layout: 2,
        });
}
