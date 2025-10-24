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
}

impl Graphics {
    pub fn new(node: &Gd<Node>) -> Self {
        Self {
            sprite: node.get_node_as::<Sprite2D>("Sprite2D"),
            animation_player: node.get_node_as::<AnimationPlayer>("AnimationPlayer"),
        }
    }

    pub fn update<T: State<impl IntoStateMachine> + Display>(
        &mut self,
        state: &T,
        dir: &Direction,
    ) {
        let anim = format!("{}_{}", state, dir);
        self.animation_player.play_ex().name(&anim).done();
    }

    pub fn get_animation_length(&self, name: &str) -> f64 {
        let Some(anim) = self.animation_player.get_animation(name) else {
            return 0.0;
        };
        anim.get_length() as f64
    }
}
