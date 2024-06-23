use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use leafwing_input_manager::prelude::*;
use seldom_state::prelude::*;
use std::any::type_name;

use crate::characters::{state::Attack, Animations, Toward};


pub fn attacked(
    In(entity): In<Entity>,
    mut events: EventReader<AnimationEvent>,
    player_attacked: Query<(&Animations, &SpritesheetAnimation), With<Attack>>,
) -> bool {
    let (ids, animation) = player_attacked.get(entity).unwrap();

    for event in events.read() {
        if let AnimationEvent::AnimationEnd { animation_id, .. } = event {

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


pub fn attack<A: Actionlike>(action: A) -> impl Trigger<Out = Result<Toward, ()>> {
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