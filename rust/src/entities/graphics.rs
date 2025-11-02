use godot::{
    classes::{AnimationPlayer, Node, Sprite2D},
    obj::Gd,
};
use statig::{IntoStateMachine, blocking::State};
use std::fmt::Display;

use crate::entities::movements::Direction;

#[derive(Clone)]
pub struct Graphics {
    sprite: Gd<Sprite2D>,
    pub animation_player: Gd<AnimationPlayer>,
    previous_animation: String,
}

impl Graphics {
    pub fn new(node: &Gd<Node>) -> Self {
        Self {
            sprite: node.get_node_as::<Sprite2D>("Sprite2D"),
            animation_player: node.get_node_as::<AnimationPlayer>("AnimationPlayer"),
            previous_animation: "".to_string(),
        }
    }

    pub fn update<T: State<impl IntoStateMachine> + Display + PartialEq>(
        &mut self,
        state: &T,
        dir: &Direction,
    ) {
        // Do not have an animation for casting a spell.
        if state.to_string() == "cast_spell" {
            // TODO: Tween player sprite or add shader for casting spells.
            return;
        }
        let anim = format!("{}_{}", state, dir);
        if anim != self.previous_animation {
            self.animation_player.play_ex().name(&anim).done();
            self.previous_animation = anim;
        }
    }

    pub fn play_then_resume(&mut self, str: &str) {
        let cur = self.animation_player.get_current_animation().to_string();
        self.animation_player.play_ex().name(str).done();
        self.animation_player.queue(&cur);
    }

    pub fn get_animation_length(&self, name: &str) -> f64 {
        let Some(anim) = self.animation_player.get_animation(name) else {
            return 0.0;
        };
        anim.get_length() as f64
    }
}
