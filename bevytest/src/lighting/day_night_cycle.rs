//! This module creates a simple day/night cycle. This amounts to a
//! rotating directional light for the sun, which currently doesn't
//! change wavelength.

use bevy::prelude::*;

use std::f32::consts::TAU;

const SECONDS_PER_DAY: f32 = 24.0 * 3600.0;

#[derive(Clone, Copy, Debug, Resource)]
pub struct Settings {
    /// How fast to spin the sun, defaults to 3600.0 or approximately
    /// 1 hour per second on earth (during an equinox on the equator)
    pub rate_multiplier: f32,
    /// The axis on which the sun should rotate, defaults to [`Vec3::X`]
    pub rotation_axis: Vec3,
    /// The directional light intensity (in lux), defaults to 50,000 lx
    pub illuminance: f32,
    /// The color of the sun, defaults to [`Color::WHITE`]
    pub color: Color,
    /// Whether the sun casts shadows, defaults to `true`.
    pub shadows: bool,
}


impl Default for Settings {
    fn default() -> Settings {
        Settings {
            rate_multiplier: 3600.0,
            rotation_axis: Vec3::X,
            color: Color::WHITE,
            illuminance: 50_000.0,
            shadows: true,
        }
    }
}

#[derive(Default)]
pub struct Lighting {
    pub settings: Settings,
}

impl Plugin for Lighting {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings)
            .add_startup_system(Lighting::add_sun)
            .add_system(Lighting::advance_time);
    }
}

#[derive(Component, Debug)]
pub struct Sun {
    rate_multiplier: f32,
    axis: Vec3,
}

#[derive(Bundle)]
pub struct SunBundle {
    directional_light_bundle: DirectionalLightBundle,
    sun: Sun,
}

impl From<&Settings> for SunBundle {
    fn from(settings: &Settings) -> SunBundle {
        let directional_light = DirectionalLight {
            color: settings.color,
            illuminance: settings.illuminance,
            shadows_enabled: settings.shadows,
            ..default()
        };
        
        let directional_light_bundle = DirectionalLightBundle {
            directional_light,
            ..default()
        };

        let sun = Sun {
            rate_multiplier: settings.rate_multiplier,
            axis: settings.rotation_axis
        };
        
        SunBundle {
            directional_light_bundle,
            sun,
        }
    }
}

impl Lighting {
    fn add_sun(mut commands: Commands, settings: Res<Settings>) {
        commands.spawn(SunBundle::from(settings.as_ref()))
            .insert(Name::new("Sun"));
    }

    fn advance_time(
        time: Res<Time>,
        mut query: Query<(&Sun, &DirectionalLight, &mut Transform)>
    ) {

        // Basic time of day assuming equatorial lighting at equinox for simplicity.

        for (sun, _light, mut transform) in &mut query {
            let seconds_passed = time.elapsed_seconds() * sun.rate_multiplier;
            let rotation_radians = TAU * seconds_passed / SECONDS_PER_DAY;
            let rotation_quat = Quat::from_axis_angle(sun.axis, rotation_radians);

            transform.rotation = rotation_quat;
        }
    }
}
