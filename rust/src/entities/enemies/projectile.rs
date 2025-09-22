use godot::{
    classes::{Node2D, Timer},
    prelude::*,
};

use crate::{
    entities::{
        damage::{AttackData, Damage, DamageType, ElementType},
        entity_hitbox::Hurtbox,
    },
    utils::collision_layers,
};

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Projectile {
    pub velocity: Vector2,
    #[init(node = "Hurtbox")]
    pub hurtbox: OnReady<Gd<Hurtbox>>,
    start_pos: Vector2,
    pub speed: real,
    #[init(val = OnReady::manual())]
    timer: OnReady<Gd<Timer>>,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Projectile {
    fn ready(&mut self) {
        self.speed = 500.0;
        self.start_pos = self.base().get_position();
        self.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.hurtbox.clone(),
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Elemental(ElementType::Fire),
            },
        });
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(2.0);
        self.base_mut().add_child(&timer);

        self.timer.init(timer);

        self.timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_timer_timeout);
        self.timer.start();
    }

    fn process(&mut self, delta: f64) {
        let velocity = self.velocity;
        let position = self.base().get_position();
        self.base_mut()
            .set_position(position + velocity * delta as f32);
    }
}

#[godot_api]
impl Projectile {
    #[signal]
    fn contacted_player();

    fn on_timer_timeout(&mut self) {
        self.base_mut().queue_free();
    }

    pub fn on_parried(&mut self) {
        self.timer.start();
        let cur_pos = self.base().get_position();
        self.velocity = cur_pos.direction_to(self.start_pos) * self.speed;

        let mut area = self.base().get_node_as::<Hurtbox>("Hurtbox");
        area.set_collision_layer_value(
            collision_layers::CollisionLayers::EnemyHurtbox as i32,
            false,
        );
        area.set_collision_mask_value(
            collision_layers::CollisionLayers::PlayerHitbox as i32,
            false,
        );
        area.set_collision_layer_value(
            collision_layers::CollisionLayers::PlayerHurtbox as i32,
            true,
        );
        area.set_collision_mask_value(collision_layers::CollisionLayers::EnemyHitbox as i32, true);
    }
}
