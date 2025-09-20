use godot::{
    builtin::Vector2,
    classes::{Area2D, IStaticBody2D, Node2D, StaticBody2D, class_macros::registry::class},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use crate::entities::{damage::Damageable, entity_hitbox::EntityHitbox, movements};

#[derive(GodotClass)]
#[class(init, base = StaticBody2D)]
pub struct BounceEnemy {
    #[init(val = Vector2::UP)]
    velocity: Vector2,
    #[init(val = Vector2::new(934.0, -330.0))]
    target: Vector2,
    #[init(node = "EntityHitbox")]
    hitbox: OnReady<Gd<EntityHitbox>>,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for BounceEnemy {
    // fn ready(&mut self) {
    //     self.hitbox
    //         .signals()
    //         .area_entered()
    //         .connect_other(&self.to_gd(), Self::on_hitbox_entered_player_hitbox);
    // }
    fn physics_process(&mut self, delta: f32) {
        movements::move_bounce(self, self.velocity, 50.0, delta);
    }
}

#[godot_api]
impl BounceEnemy {
    // fn on_hitbox_entered_player_hitbox(&mut self, area: Gd<Area2D>) {
    //     let mut a = area.cast::<EntityHitbox>();
    //     println!("Player takign damage");
    //     a.take_damage(20);
    // }
}

impl movements::Moveable for BounceEnemy {
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}
