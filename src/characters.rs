use bevy::prelude::*;
use player::PlayerPlugin;
use leafwing_input_manager::prelude::*;

pub mod player;

#[derive(Actionlike, Clone, Eq, Hash, PartialEq, Reflect)]
enum Action {
    Idle,
    Jump,
}

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin);
    }
}
