use godot::{
    classes::{Area2D, Node2D, Timer},
    prelude::*,
};

use crate::{entities::hit_reg::Hurtbox, utils::collision_layers};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Projectile {
    pub velocity: Vector2,
    #[init(node = "Hurtbox")]
    pub hurtbox: OnReady<Gd<Hurtbox>>,
    pub target: Vector2,
    start_pos: Vector2,
    speed: real,
    #[init(node = "Timer")]
    timer: OnReady<Gd<Timer>>,
    was_parried: bool,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Projectile {
    fn ready(&mut self) {
        self.hurtbox
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_area_entered);
        self.hurtbox
            .signals()
            .area_exited()
            .connect_other(&self.to_gd(), Self::on_area_exited);
        self.speed = 500.0;
        self.start_pos = self.base().get_position();
    }

    fn process(&mut self, delta: f64) {
        let position = self.base().get_position();
        let velocity = position.direction_to(self.target) * self.speed;
        self.base_mut()
            .set_position(position + velocity * delta as f32);
    }
}

#[godot_api]
impl Projectile {
    fn on_area_entered(&mut self, _area: Gd<Area2D>) {
        if !self.was_parried {
            self.run_deferred_gd(|mut this| this.queue_free());
        }
    }

    fn on_area_exited(&mut self, _area: Gd<Area2D>) {
        if self.was_parried {
            self.was_parried = false;
        }
    }

    pub fn on_parried(&mut self) {
        self.was_parried = true;
        self.timer.start();
        self.target = self.start_pos;

        self.hurtbox.set_collision_layer_value(
            collision_layers::CollisionLayers::EnemyHurtbox as i32,
            false,
        );
        self.hurtbox.set_collision_mask_value(
            collision_layers::CollisionLayers::PlayerHitbox as i32,
            false,
        );
        self.hurtbox.set_collision_layer_value(
            collision_layers::CollisionLayers::PlayerHurtbox as i32,
            true,
        );
        self.hurtbox
            .set_collision_mask_value(collision_layers::CollisionLayers::EnemyHitbox as i32, true);
    }
}
