use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use player::PlayerPlugin;
use bevy_spritesheet_animation::prelude::*;
use serde::{Deserialize, Serialize};



pub mod player;

pub mod state;

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

#[derive(Default, Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Reflect, Component)]
pub enum Toward {
    #[default]
    Up,
    Down,
    Left,
    Right,
}


#[derive(Component)]
pub struct Animations {
    pub idle: Animation,
    pub run: Animation,
    pub attack: Animation,
}

pub struct Animation {
    pub up: AnimationId,
    pub down: AnimationId,
    pub left: AnimationId,
    pub right: AnimationId,
}

impl Animation {
    pub fn new_clips<F>(
        library: &mut ResMut<SpritesheetLibrary>,
        up_builder: F,
        down_builder: F,
        left_builder: F,
        right_builder: F,
    ) -> Self
    where
        F: Fn(&mut bevy_spritesheet_animation::clip::AnimationClip),
    {
        let up_id = library.new_clip(up_builder);
        let down_id = library.new_clip(down_builder);
        let left_id = library.new_clip(left_builder);
        let right_id = library.new_clip(right_builder);

        Self {
            up: library.new_animation(|animation| {
                animation.add_stage(up_id.into());
            }),
            down: library.new_animation(|animation| {
                animation.add_stage(down_id.into());
            }),
            left: library.new_animation(|animation| {
                animation.add_stage(left_id.into());
            }),
            right: library.new_animation(|animation| {
                animation.add_stage(right_id.into());
            }),
        }
    }

    pub fn new_clips_with_repeat<F>(
        library: &mut ResMut<SpritesheetLibrary>,
        up_builder: F,
        down_builder: F,
        left_builder: F,
        right_builder: F,
        repeat: AnimationRepeat,
    ) -> Self
    where
        F: Fn(&mut bevy_spritesheet_animation::clip::AnimationClip),
    {
        let up_id = library.new_clip(up_builder);
        let down_id = library.new_clip(down_builder);
        let left_id = library.new_clip(left_builder);
        let right_id = library.new_clip(right_builder);

        Self {
            up: library.new_animation(|animation| {
                animation.add_stage(up_id.into()).set_repeat(repeat);
            }),
            down: library.new_animation(|animation| {
                animation.add_stage(down_id.into()).set_repeat(repeat);
            }),
            left: library.new_animation(|animation| {
                animation.add_stage(left_id.into()).set_repeat(repeat);
            }),
            right: library.new_animation(|animation| {
                animation.add_stage(right_id.into()).set_repeat(repeat);
            }),
        }
    }
}
