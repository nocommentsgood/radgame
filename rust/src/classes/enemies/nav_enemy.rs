use godot::{
    classes::{CharacterBody2D, ICharacterBody2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
struct NavEnemy {
    #[init(val = 200.0)]
    speed: f32,
    #[export]
    target_pos: Vector2,
    base: Base<CharacterBody2D>,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<godot::classes::NavigationAgent2D>>,
}

#[godot_api]
impl ICharacterBody2D for NavEnemy {
    fn ready(&mut self) {
        self.nav_agent.set_path_desired_distance(4.0);
        self.nav_agent.set_target_desired_distance(4.0);
        self.nav_agent
            .signals()
            .velocity_computed()
            .connect_other(&self.to_gd(), Self::on_velocity_computed);

        self.apply_deferred(|this| this.setup());
    }

    fn physics_process(&mut self, _delta: f32) {
        if self.nav_agent.is_navigation_finished() {
            return;
        }

        let next = self.nav_agent.get_next_path_position();
        let v = self.base().get_global_position().direction_to(next) * self.speed;
        if self.nav_agent.get_avoidance_enabled() {
            self.nav_agent.set_velocity(v);
        }
    }
}

#[godot_api]
impl NavEnemy {
    fn on_velocity_computed(&mut self, safe_vel: Vector2) {
        self.base_mut().set_velocity(safe_vel);
        self.base_mut().move_and_slide();
    }

    fn setup(&mut self) {
        let tree = self.base().get_tree().unwrap();
        godot::task::spawn(async move {
            tree.signals().physics_frame().to_future().await;
        });
        self.set_movement_target(self.target_pos);
    }

    fn set_movement_target(&mut self, target: Vector2) {
        self.nav_agent.set_target_position(target);
    }
}
