use bevy::{
    prelude::*,
};
use bevytest::prelude::*;
use earth::{
    prelude::*,
    grid::hex::*,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    let settings = SceneSettings {
        cameras: CameraSettings::AzimuthElevation(
            AzimuthElevationSettings::high_angle()
        ),
        lighting: LightSettings::DayNightCycle(
            DayNightCycleSettings::default()
        ),
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScenePlugin::with_settings(settings))
        .add_plugin(PerformanceMonitorPlugin::with_font("fonts/source_code_pro/SourceCodePro-Regular.otf"))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(EarthPlugins)
        .add_startup_system(spawn_ocean)
        .run();
}

fn spawn_ocean(mut commands: Commands) {
    let position = GridVec::ZERO;
    commands.add(ocean::AddOcean {
        grid_position: position,
        ..default()
    });

    commands.add(ocean::AddOcean {
        grid_position: position + GridVec::NORTH,
        ..default()
    });

    commands.add(ocean::AddOcean {
        grid_position: position + GridVec::NORTHEAST,
        ..default()
    });

    commands.add(ocean::AddOcean {
        grid_position: position + GridVec::SOUTHEAST,
        ..default()
    });
}
