use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use characters::{
    player::{create_player, PlayerType}, CharactersPlugin, Toward,
};
use seldom_state::StateMachinePlugin;
pub mod characters;
pub mod state_machine;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        // Add the plugin to enable animations.
        // This makes the SpritesheetLibrary resource available to your systems.
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(StateMachinePlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CharactersPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut library: ResMut<SpritesheetLibrary>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());
    create_player(
        &mut commands,
        &asset_server,
        &mut library,
        &mut texture_atlas_layouts,
        PlayerType::Adventurer,
        Toward::Up,
        Vec3::new(100., 100., 0.),
    );
}
