use godot::{
    classes::{AnimationPlayer, Node, Sprite2D},
    obj::Gd,
};
use statig::{IntoStateMachine, blocking::State};
use std::fmt::Display;

use crate::entities::movements::Direction;
use std::sync::atomic;

pub struct ID(i64);

impl ID {
    pub fn new() -> Self {
        Self(BUMP.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

static BUMP: atomic::AtomicI64 = atomic::AtomicI64::new(4);

pub struct Entity {
    id: ID,
    graphics: Graphics,
}

impl Entity {
    fn new(node: &Gd<Node>) -> Self {
        Self {
            id: ID::new(),
            graphics: Graphics::new(node),
        }
    }

    pub fn id(&self) -> &ID {
        &self.id
    }
}

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
}
