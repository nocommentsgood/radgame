use godot::prelude::*;

use crate::classes::components::hurtbox::Hurtbox;
use crate::classes::components::timer_component::Timer;
use crate::utils::collision_layers;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Projectile {
    pub velocity: Vector2,
    start_pos: Vector2,
    speed: real,
    timeout: Timer,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Projectile {
    fn ready(&mut self) {
        self.speed = 500.0;
        self.start_pos = self.base().get_position();
        self.timeout = Timer::new(3.0);
    }

    fn process(&mut self, delta: f64) {
        self.tick();
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

    pub fn on_parried(&mut self) {
        self.timeout.reset();
        let cur_pos = self.base().get_position();
        self.velocity = cur_pos.direction_to(self.start_pos) * self.speed;

        let mut area = self.base().get_node_as::<Hurtbox>("Hurtbox");
        area.set_collision_layer_value(
            collision_layers::CollisionLayers::EnemyHurtbox.into(),
            false,
        );
        area.set_collision_layer_value(
            collision_layers::CollisionLayers::PlayerHurtbox.into(),
            true,
        );
    }

    fn tick(&mut self) {
        let delta = self.base().get_process_delta_time() as f32;
        self.timeout.value -= delta;

        if self.timeout.value <= 0.0 {
            self.base_mut().queue_free();
        }
    }
}
