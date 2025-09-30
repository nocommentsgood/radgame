use godot::{
    classes::{AnimationPlayer, Node, Sprite2D},
    obj::Gd,
};
use statig::{IntoStateMachine, blocking::State};
use std::fmt::Display;

use crate::entities::movements::Direction;

#[derive(Clone)]
pub struct EntGraphics {
    sprite: Gd<Sprite2D>,
    pub animation_player: Gd<AnimationPlayer>,
}

impl EntGraphics {
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
}
