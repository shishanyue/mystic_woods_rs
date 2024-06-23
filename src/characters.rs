use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;
use player::PlayerPlugin;
pub mod player;

#[derive(Actionlike, Clone, Eq, Hash, PartialEq, Reflect)]
pub enum CharacterAction {
    Idle,
    Move,
    Jump,
    Attack,
}

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin);
    }
}
