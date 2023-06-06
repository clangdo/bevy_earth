
// Test - Procedural Generation
//
//

use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy::{
    prelude::*,
};
use bevytest::prelude::*;
use earth::prelude::*;

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
        .add_plugin(
            PerformanceMonitorPlugin::with_font("fonts/source_code_pro/SourceCodePro-Regular.otf")
        ) // Add a performance monitor plugin with a specific font
        .add_plugin(WorldInspectorPlugin::new()) // Add a world inspector plugin
        .add_plugins(EarthPlugins) // Add custom Earth plugins
        .add_startup_system(proc_gen) // Run the proc_gen system after the add_grid system
        .run(); // Run the application
}

fn proc_gen(mut commands: Commands) {
    commands.add(ScheduleGenerate);
}

