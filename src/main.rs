use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_spritesheet_animation::prelude::*;
use characters::CharactersPlugin;
use seldom_state::StateMachinePlugin;

pub mod characters;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the plugin to enable animations.
        // This makes the SpritesheetLibrary resource available to your systems.
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(StateMachinePlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CharactersPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
