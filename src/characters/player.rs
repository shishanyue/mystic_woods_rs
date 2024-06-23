use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use leafwing_input_manager::{orientation::Rotation, prelude::*};
use seldom_state::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::type_name;

use super::CharacterAction;

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
    pub animation_ids: PlayerAnimationIds,
    pub rigid_body: RigidBody,
    pub controller: KinematicCharacterController,
    pub velocity:Velocity
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
        }).insert(KinematicCharacterControllerOutput::default())
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
                        transform: Transform {
                            translation,
                            scale: Vec3::new(2., 2., 0.),
                            ..Default::default()
                        },
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
                        .trans_builder(attack(CharacterAction::Attack), |_: &Direction, toward| {
                            Some(Attack { toward })
                        })
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
                    rigid_body: RigidBody::KinematicPositionBased,
                    controller: KinematicCharacterController::default(),
                    velocity:Velocity { linvel: Vec2::new(2., 3.), angvel: 0. }
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
            .add_systems(Update, (walk,read_result_system, player_animation))
            .register_type::<CharacterAction>()
            .register_type::<Direction>()
            .register_type::<Toward>();
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

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Direction {
    val: Vec3,
}

impl Default for Direction {
    fn default() -> Self {
        Self::ZERO
    }
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

    #[inline(always)]
    #[must_use]
    pub const fn from_toward(toward: &Toward) -> Self {
        match toward {
            Toward::Up => Direction::UP,
            Toward::Down => Direction::DOWN,
            Toward::Left => Direction::LEFT,
            Toward::Right => Direction::RIGHT,
        }
    }
}

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Attack {
    toward: Toward,
}

#[derive(Component)]
pub struct PlayerAnimationIds {
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
                    .set_default_duration(AnimationDuration::PerCycle(1500));
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

fn walk(mut groundeds: Query<(&mut KinematicCharacterController, &Direction)>, time: Res<Time>) {
    for (mut controller , direction) in &mut groundeds {
        controller.translation = Some(Vec2 {
            x: direction.val.x * time.delta_seconds() * PLAYER_SPEED,
            y: direction.val.y * time.delta_seconds() * PLAYER_SPEED,
        });
    }
}


fn read_result_system(controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    for (entity, output) in controllers.iter() {
        println!(
            "Entity {:?} moved by {:?} and touches the ground: {:?}",
            entity, output.effective_translation, output.grounded
        );
    }
}

fn attacked(
    In(entity): In<Entity>,
    mut events: EventReader<AnimationEvent>,
    player_attacked: Query<(&PlayerAnimationIds, &SpritesheetAnimation), With<Attack>>,
) -> bool {
    let (ids, animation) = player_attacked.get(entity).unwrap();

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
                return true;
            }
        }
    }
    false
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
