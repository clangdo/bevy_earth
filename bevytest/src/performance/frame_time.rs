//! This module implements a simple frame time UI element useful for
//! testing.  Some of the functionality herin is based on the
//! log_diagnostics [bevy
//! example](https://github.com/bevyengine/bevy/blob/920543c824735dc1df6f4a59e7036e653dd5a553/examples/diagnostics/log_diagnostics.rs).
//! Much of the code is based on the [bevy text
//! example](https://github.com/bevyengine/bevy/blob/920543c824735dc1df6f4a59e7036e653dd5a553/examples/ui/text.rs).
//! Additional references included the [bevy ui
//! example](https://github.com/bevyengine/bevy/blob/920543c824735dc1df6f4a59e7036e653dd5a553/examples/ui/ui.rs)

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::PerformanceMonitorSettings;

pub struct FrameTimePlugin;

impl Plugin for FrameTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_systems((insert_text_styles.in_base_set(StartupSet::PreStartup), create_display))
            .add_systems((display_frame_time, display_frame_rate));
    }
}

#[derive(Component, Clone, Debug)]
struct FrameTimeDisplay;

#[derive(Component, Clone, Debug)]
struct FrameRateDisplay;

#[derive(Resource, Clone, Debug)]
struct DebugTextStyles {
    normal: TextStyle,
    highlight: TextStyle,
}

fn insert_text_styles(
    mut commands: Commands,
    settings: Res<PerformanceMonitorSettings>,
    assets: Res<AssetServer>
) {
    let font = assets.load(settings.font_path.clone());

    commands.insert_resource(DebugTextStyles{
        normal: TextStyle {
            font: font.clone(),
            font_size: 20.0,
            color: Color::WHITE,
        },

        highlight: TextStyle {
            font,
            font_size: 20.0,
            color: Color::GREEN,
        },
    });
}

fn create_display(
    mut commands: Commands,
    styles: Res<DebugTextStyles>
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                margin: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn(assemble_frame_time_text(&styles))
                .insert(FrameTimeDisplay)
                .insert(Name::new("Frame Time Display"));

            builder
                .spawn(assemble_frame_rate_text(&styles))
                .insert(FrameRateDisplay)
                .insert(Name::new("Frame Rate Display"));
        })
        .insert(Name::new("Frame Rate UI"));


}

fn assemble_frame_rate_text(styles: &DebugTextStyles) -> TextBundle {
    let size = styles.highlight.font_size.max(styles.normal.font_size);
    TextBundle::from_sections([
        TextSection::new("?", styles.highlight.clone()),
        TextSection::new(" frames per second", styles.normal.clone()),
    ]).with_style(Style {
        size: Size::new(Val::Undefined, Val::Px(size)),
        ..default()
    })
}

fn assemble_frame_time_text(styles: &DebugTextStyles) -> TextBundle {
    let size = styles.highlight.font_size.max(styles.normal.font_size);
    TextBundle::from_sections([
        TextSection::new("?", styles.highlight.clone()),
        TextSection::new(" ms average frame time", styles.normal.clone()),
    ]).with_style(Style {
        size: Size::new(Val::Undefined, Val::Px(size)),
        ..default()
    })
}

fn display_frame_time(
    diagnostics: Res<Diagnostics>,
    mut displays: Query<&mut Text, With<FrameTimeDisplay>>
) {
    let frame_times = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .expect("no frame time diagnostics for frame time display plugin");

    let frame_time_average = match frame_times.average() {
        Some(average) => format!("{average:.1}"),
        None => "?".into(),
    };

    for mut display in displays.iter_mut() {
        display.sections[0].value = frame_time_average.clone();
    }
}

fn display_frame_rate(
    diagnostics: Res<Diagnostics>,
    mut displays: Query<&mut Text, With<FrameRateDisplay>>
) {
    let frame_rates = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS)
        .expect("no frame rate diagnostics for frame time display plugin");

    let frame_rate_average = match frame_rates.average() {
        Some(average) => format!("{average:.1}"),
        None => "?".into(),
    };

    for mut display in displays.iter_mut() {
        display.sections[0].value = frame_rate_average.clone();
    }
}
