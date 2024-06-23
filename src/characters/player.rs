use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use leafwing_input_manager::{orientation::Rotation, prelude::*};
use seldom_state::prelude::*;

use crate::state_machine::trigger::{attack, attacked};

use super::{
    state::{Attack, Direction},
    Animation, Animations, CharacterAction, Toward,
};

#[derive(Default, Component, Clone, Copy)]
pub enum PlayerType {
    #[default]
    Adventurer,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player_name: Name,
    pub player_type: PlayerType,
    pub player_marker: Player,
    pub sprite_sheet: SpriteSheetBundle,
    pub input_manager: InputManagerBundle<CharacterAction>,
    pub state_machine: StateMachine,
    pub direction: Direction,
    pub toward: Toward,
    pub spritesheet_animation: SpritesheetAnimation,
    pub animation_ids: Animations,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
}

pub fn create_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    library: &mut ResMut<SpritesheetLibrary>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    player_type: PlayerType,
    default_toward: Toward,
    translation: Vec3,
) -> Entity {
    commands
        .spawn(PlayerBundle::new(
            asset_server,
            library,
            texture_atlas_layouts,
            player_type,
            default_toward,
            translation,
        ))
        .with_children(|player| match player_type {
            PlayerType::Adventurer => {
                player
                    .spawn(Collider::cuboid(8., 10.))
                    .insert(TransformBundle::from(Transform::from_xyz(0., -8., 0.)));
            }
        })
        .insert(KinematicCharacterControllerOutput::default())
        .id()
}

impl PlayerBundle {
    pub fn new(
        asset_server: &Res<AssetServer>,
        library: &mut ResMut<SpritesheetLibrary>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        player_type: PlayerType,
        default_toward: Toward,
        translation: Vec3,
    ) -> Self {
        match player_type {
            PlayerType::Adventurer => {
                let player_image = asset_server.load("sprites/characters/player.png");

                let altas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    Vec2::new(48., 48.),
                    6,
                    10,
                    None,
                    None,
                ));

                let animation = init_player_animation(library);

                PlayerBundle {
                    player_name: Name::new("Player"),
                    player_type: PlayerType::Adventurer,
                    sprite_sheet: SpriteSheetBundle {
                        texture: player_image,
                        atlas: TextureAtlas {
                            layout: altas_layout,
                            index: 0,
                        },
                        transform: Transform::from_translation(translation),
                        ..Default::default()
                    },
                    input_manager: InputManagerBundle {
                        input_map: InputMap::default()
                            .insert(CharacterAction::Move, VirtualDPad::arrow_keys())
                            .insert(CharacterAction::Move, VirtualDPad::wasd())
                            .insert(CharacterAction::Attack, KeyCode::KeyJ)
                            .build(),
                        ..default()
                    },
                    state_machine: StateMachine::default()
                        .trans::<Direction, _>(attack(CharacterAction::Attack), Attack)
                        .trans::<Attack, _>(attacked, Direction::ZERO)
                        .trans_builder(
                            axis_pair(
                                CharacterAction::Move,
                                0.0..f32::INFINITY,
                                Rotation::NORTH..Rotation::NORTH,
                            ),
                            |_: &Direction, axis_pair| {
                                Some(Direction {
                                    val: Vec3 {
                                        x: axis_pair.x(),
                                        y: axis_pair.y(),
                                        z: 0.,
                                    },
                                })
                            },
                        ),
                    direction: Direction::from_toward(&default_toward),
                    toward: default_toward,
                    spritesheet_animation: SpritesheetAnimation::from_id(animation.idle.up),
                    animation_ids: animation,
                    player_marker: Player,
                    rigid_body: RigidBody::KinematicVelocityBased,
                    velocity: Velocity::zero(),
                }
            }
        }
    }
}

#[derive(Default, Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CharacterAction>::default())
            .add_systems(Update, (walk, player_animation))
            .register_type::<CharacterAction>()
            .register_type::<Direction>()
            .register_type::<Toward>();
    }
}

fn init_player_animation(library: &mut ResMut<SpritesheetLibrary>) -> Animations {
    use bevy_spritesheet_animation::clip::AnimationClip;
    let clips_builder = |columns, rows, play_row, play_column_range| {
        move |clip: &mut AnimationClip| match play_column_range {
            Some(range) => {
                clip.push_frame_indices(
                    Spritesheet::new(columns, rows).row_partial(play_row, range),
                );
            }
            None => {
                clip.push_frame_indices(Spritesheet::new(columns, rows).row(play_row))
                    .set_default_duration(AnimationDuration::PerCycle(1000));
            }
        }
    };

    Animations {
        idle: Animation::new_clips(
            library,
            clips_builder(6, 10, 2, None),
            clips_builder(6, 10, 0, None),
            clips_builder(6, 10, 1, None),
            clips_builder(6, 10, 1, None),
        ),
        run: Animation::new_clips(
            library,
            clips_builder(6, 10, 5, None),
            clips_builder(6, 10, 3, None),
            clips_builder(6, 10, 4, None),
            clips_builder(6, 10, 4, None),
        ),
        attack: Animation::new_clips_with_repeat(
            library,
            clips_builder(6, 10, 8, Some(..4)),
            clips_builder(6, 10, 6, Some(..4)),
            clips_builder(6, 10, 7, Some(..4)),
            clips_builder(6, 10, 7, Some(..4)),
            AnimationRepeat::Cycles(1),
        ),
    }
}

const PLAYER_SPEED: f32 = 100.;

fn walk(mut groundeds: Query<(&mut Velocity, &Direction)>) {
    for (mut velocity, direction) in &mut groundeds {
        velocity.linvel = Vec2 {
            x: direction.val.x * PLAYER_SPEED,
            y: direction.val.y * PLAYER_SPEED,
        };
    }
}

fn player_animation(
    mut players: Query<(
        &mut SpritesheetAnimation,
        &Animations,
        Option<&Direction>,
        Option<&Attack>,
        &mut Sprite,
        &mut Toward,
        &mut Velocity
    )>,
) {
    for (mut animation, ids, direction, attack, mut sprite, mut toward,mut velocity) in &mut players {
        if let Some(direction) = direction {
            let direction = direction.val;
            if direction.x > 0. {
                sprite.flip_x = false;
            }
            if direction.y > 0. && direction.x == 0. && animation.animation_id != ids.run.up {
                *toward = Toward::Up;
                animation.animation_id = ids.run.up;
            }
            if direction.y < 0. && direction.x == 0. && animation.animation_id != ids.run.down {
                *toward = Toward::Down;
                animation.animation_id = ids.run.down;
            }
            if direction.y == 0. && direction.x > 0. && animation.animation_id != ids.run.right {
                *toward = Toward::Right;
                animation.animation_id = ids.run.right;
            }
            if direction.y == 0. && direction.x < 0. && animation.animation_id != ids.run.left {
                *toward = Toward::Left;
                sprite.flip_x = true;
                animation.animation_id = ids.run.left;
            }
            if direction.y == 0.
                && direction.x == 0.
                && animation.animation_id != ids.idle.up
                && animation.animation_id != ids.idle.down
                && animation.animation_id != ids.idle.left
                && animation.animation_id != ids.idle.right
            {
                match *toward {
                    Toward::Up => animation.animation_id = ids.idle.up,
                    Toward::Down => animation.animation_id = ids.idle.down,
                    Toward::Left => animation.animation_id = ids.idle.left,
                    Toward::Right => animation.animation_id = ids.idle.right,
                }
            }
        }
        if attack.is_some() {
            velocity.linvel = Vec2::ZERO;

            match *toward {
                Toward::Up => {
                    if animation.animation_id != ids.attack.up {
                        animation.animation_id = ids.attack.up;
                    }
                }
                Toward::Down => {
                    if animation.animation_id != ids.attack.down {
                        animation.animation_id = ids.attack.down;
                    }
                }
                Toward::Left => {
                    if animation.animation_id != ids.attack.left {
                        animation.animation_id = ids.attack.left;
                    }
                }
                Toward::Right => {
                    if animation.animation_id != ids.attack.right {
                        animation.animation_id = ids.attack.right;
                    }
                }
            }
        }
    }
}
