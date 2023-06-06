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
        lighting: LightSettings::DayNightCycle(DayNightCycleSettings{
            rate_multiplier: 100.0,
            color: Color::rgb(1.0, 0.9, 0.7),
            ..default()
        }),
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScenePlugin::with_settings(settings))
        .add_plugin(PerformanceMonitorPlugin::with_font("fonts/source_code_pro/SourceCodePro-Regular.otf"))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(EarthPlugins)
        .add_startup_systems((add_grid, spawn_nature).chain())
        .run();
}


fn add_grid(mut commands: Commands) {
    commands.insert_resource(Grid::default());
}

fn spawn_nature(mut commands: Commands) {
    let position = GridVec::ZERO;
    commands.add(nature::AddForest {
        grid_position: position,
    });

    commands.add(nature::AddForest {
        grid_position: position + GridVec::NORTH,
    });

    commands.add(nature::AddForest {
        grid_position: position + GridVec::NORTHEAST,
    });

    commands.add(nature::AddForest {
        grid_position: position + GridVec::SOUTHEAST,
    });
}
