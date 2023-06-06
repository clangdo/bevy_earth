use bevy::prelude::*;
use earth::prelude::*;

mod editor;
use editor::EditorPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EarthPlugins)
        .add_plugins(EditorPlugins)
        .run();
}
