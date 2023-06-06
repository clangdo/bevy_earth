use bevy::prelude::*;

/// This plugin simply changes the clear color to a reasonable light
/// blue.
///
/// In the future this could do much more to create a reasonable
/// looking sky.
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::ALICE_BLUE));
    }
}
