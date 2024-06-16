use std::any::type_name;

use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use leafwing_input_manager::{orientation::Rotation, prelude::*};
use seldom_state::{prelude::*, trigger::TriggerOut};
use serde::{Deserialize, Serialize};


#[derive(Component)]
pub enum PlayerType {
    Adventurer
}


pub struct PlayerBundle{
    pub player_type:PlayerType,
    pub player_marker:Player
}





#[derive(Default, Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(PreStartup, setup)
            .add_systems(Update, (walk, player_animation))
            .register_type::<Action>()
            .register_type::<Direction>()
            .register_type::<Toward>();
    }
}

#[derive(Actionlike, Clone, Eq, Hash, PartialEq, Reflect)]
enum Action {
    Move,
    Attack,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Reflect, Component)]
enum Toward {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Direction {
    val: Vec3,
}

impl Direction {
    /// All zeroes.
    pub const ZERO: Self = Self::splat(0.0);

    /// A unit vector pointing along the positive X axis.
    pub const RIGHT: Self = Self::new(1.0, 0.0, 0.0);

    /// A unit vector pointing along the positive Y axis.
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);

    /// A unit vector pointing along the negative X axis.
    pub const LEFT: Self = Self::new(-1.0, 0.0, 0.0);

    /// A unit vector pointing along the negative Y axis.
    pub const DOWN: Self = Self::new(0.0, -1.0, 0.0);

    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self {
            val: Vec3::splat(v),
        }
    }

    /// Creates a new direction.
    #[inline(always)]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            val: Vec3::new(x, y, z),
        }
    }
}

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Attack {
    toward: Toward,
}

#[derive(Component)]
struct PlayerAnimationIds {
    idle: Animation2dIds,
    run: Animation2dIds,
    attack: Animation2dIds,
}

struct Animation2dIds {
    up: AnimationId,
    down: AnimationId,
    left: AnimationId,
    right: AnimationId,
}

impl Animation2dIds {
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

        Animation2dIds {
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

        Animation2dIds {
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

fn init_player_animation(library: &mut ResMut<SpritesheetLibrary>) -> PlayerAnimationIds {
    let clips_builder = |columns, rows, play_row, play_column_range| {
        move |clip: &mut bevy_spritesheet_animation::clip::AnimationClip| match play_column_range {
            Some(range) => {
                clip.push_frame_indices(
                    Spritesheet::new(columns, rows).row_partial(play_row, range),
                );
            }
            None => {
                clip.push_frame_indices(Spritesheet::new(columns, rows).row(play_row));
            }
        }
    };

    PlayerAnimationIds {
        idle: Animation2dIds::new_clips(
            library,
            clips_builder(6, 10, 2, None),
            clips_builder(6, 10, 0, None),
            clips_builder(6, 10, 1, None),
            clips_builder(6, 10, 1, None),
        ),
        run: Animation2dIds::new_clips(
            library,
            clips_builder(6, 10, 5, None),
            clips_builder(6, 10, 3, None),
            clips_builder(6, 10, 4, None),
            clips_builder(6, 10, 4, None),
        ),
        attack: Animation2dIds::new_clips_with_repeat(
            library,
            clips_builder(6, 10, 8, Some(..4)),
            clips_builder(6, 10, 6, Some(..4)),
            clips_builder(6, 10, 7, Some(..4)),
            clips_builder(6, 10, 7, Some(..4)),
            AnimationRepeat::Cycles(1),
        ),
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut library: ResMut<SpritesheetLibrary>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_image = asset_server.load("sprites/characters/player.png");

    let altas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        Vec2::new(48., 48.),
        6,
        10,
        None,
        None,
    ));

    let animation = init_player_animation(&mut library);

    commands.spawn((
        SpriteSheetBundle {
            texture: player_image,
            atlas: TextureAtlas {
                layout: altas_layout,
                index: 0,
            },
            transform: Transform::from_xyz(100., 100., 0.),
            ..Default::default()
        },
        // From `leafwing-input-manager`
        InputManagerBundle {
            input_map: InputMap::default()
                .insert(Action::Move, VirtualDPad::arrow_keys())
                .insert(Action::Move, VirtualDPad::wasd())
                .insert(Action::Attack, KeyCode::KeyJ)
                .build(),
            ..default()
        },
        StateMachine::default()
            .trans_builder(attack(Action::Attack), |_: &Direction, toward| {
                Some(Attack { toward })
            })
            .trans_builder(attacked, |_: &Attack, toward| Some(match toward {
                Toward::Up => Direction::UP,
                Toward::Down => Direction::DOWN,
                Toward::Left => Direction::LEFT,
                Toward::Right => Direction::RIGHT,
            }))
            .trans_builder(
                axis_pair(
                    Action::Move,
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
        Direction::ZERO,
        Toward::Down,
        SpritesheetAnimation::from_id(animation.idle.up),
        animation,
    ));
}

fn attack<A: Actionlike>(action: A) -> impl Trigger<Out = Result<Toward, ()>> {
    (move |In(entity): In<Entity>, actors: Query<(&ActionState<A>, &Toward)>| {
        let actor = actors.get(entity).unwrap_or_else(|_| {
            panic!(
                "entity {entity:?} with `JustPressedTrigger<{0}>` is missing `ActionState<{0}>`",
                type_name::<A>()
            )
        });

        if actor.0.just_pressed(&action) {
            Ok(*actor.1)
        } else {
            Err(())
        }
    })
    .into_trigger()
}

const PLAYER_SPEED: f32 = 200.;

fn walk(mut groundeds: Query<(&mut Transform, &Direction)>, time: Res<Time>) {
    for (mut transform, direction) in &mut groundeds {
        transform.translation += Vec3 {
            x: direction.val.x * time.delta_seconds() * PLAYER_SPEED,
            y: direction.val.y * time.delta_seconds() * PLAYER_SPEED,
            z: 0.,
        };
    }
}
fn attacked(
    In(entity): In<Entity>,
    mut events: EventReader<AnimationEvent>,
    player_attacked: Query<(&PlayerAnimationIds, &SpritesheetAnimation, &Toward), With<Attack>>,
) -> Option<Toward> {
    let (ids, animation, toward) = player_attacked.get(entity).unwrap();

    for event in events.read() {
        if let AnimationEvent::AnimationEnd { animation_id, .. } = event {
            // ... it was the main character's death animation,
            // we can go back to the main menu

            if (animation.animation_id == ids.attack.up
                || animation.animation_id == ids.attack.down
                || animation.animation_id == ids.attack.left
                || animation.animation_id == ids.attack.right)
                && animation.animation_id == *animation_id
            {
                return Some(*toward);
            }
        }
    }
    None
}

fn player_animation(
    mut players: Query<(
        &mut SpritesheetAnimation,
        &PlayerAnimationIds,
        Option<&Direction>,
        Option<&Attack>,
        &mut Sprite,
        &mut Toward,
    )>,
) {
    for (mut animation, ids, direction, attack, mut sprite, mut toward) in &mut players {
        if let Some(direction) = direction {
            let direction = direction.val;
            if direction.x >= 0. && animation.animation_id != ids.idle.left {
                sprite.flip_x = false;
            }
            if direction.y > 0. && direction.x == 0. && animation.animation_id != ids.run.up {
                *toward = Toward::Up;
                animation.animation_id = ids.run.up
            }
            if direction.y < 0. && direction.x == 0. && animation.animation_id != ids.run.down {
                *toward = Toward::Down;
                animation.animation_id = ids.run.down
            }
            if direction.y == 0. && direction.x > 0. && animation.animation_id != ids.run.right {
                *toward = Toward::Right;
                animation.animation_id = ids.run.right
            }
            if direction.y == 0. && direction.x < 0. && animation.animation_id != ids.run.left {
                *toward = Toward::Left;
                sprite.flip_x = true;
                animation.animation_id = ids.run.left
            }
            if direction.y == 0.
                && direction.x == 0.
                && animation.animation_id != ids.idle.up
                && animation.animation_id != ids.idle.down
                && animation.animation_id != ids.idle.left
                && animation.animation_id != ids.idle.right
            {
                if animation.animation_id == ids.run.up {
                    animation.animation_id = ids.idle.up
                } else if animation.animation_id == ids.run.down {
                    animation.animation_id = ids.idle.down
                } else if animation.animation_id == ids.run.left {
                    sprite.flip_x = true;
                    animation.animation_id = ids.idle.left
                } else {
                    animation.animation_id = ids.idle.right
                }
            }
        }
        if attack.is_some() {
            match *toward {
                Toward::Up => {
                    if animation.animation_id != ids.attack.up {
                        animation.animation_id = ids.attack.up
                    }
                }
                Toward::Down => {
                    if animation.animation_id != ids.attack.down {
                        animation.animation_id = ids.attack.down
                    }
                }
                Toward::Left => {
                    if animation.animation_id != ids.attack.left {
                        animation.animation_id = ids.attack.left
                    }
                }
                Toward::Right => {
                    if animation.animation_id != ids.attack.right {
                        animation.animation_id = ids.attack.right
                    }
                }
            }
        }
    }
}
