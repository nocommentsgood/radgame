use godot::{
    builtin::Vector2,
    classes::{IStaticBody2D, StaticBody2D},
    obj::{Base, Gd, OnReady},
    prelude::{GodotClass, godot_api},
};

use crate::entities::{entity_hitbox::Hitbox, movements};

#[derive(GodotClass)]
#[class(init, base = StaticBody2D)]
pub struct BounceEnemy {
    #[init(val = Vector2::UP)]
    velocity: Vector2,
    #[init(node = "EntityHitbox")]
    hitbox: OnReady<Gd<Hitbox>>,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for BounceEnemy {
    fn physics_process(&mut self, delta: f32) {
        movements::move_bounce(self, self.velocity, 50.0, delta);
    }
}

impl movements::Moveable for BounceEnemy {
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}
