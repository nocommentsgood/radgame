use godot::{
    classes::{CharacterBody2D, ICharacterBody2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
struct NavEnemy {
    #[init(val = 200.0)]
    speed: f32,
    #[init(val = Vector2::new(-172.0, 260.0))]
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

        // let t = self
        //     .base()
        //     .get_tree()
        //     .unwrap()
        //     .create_timer_ex(1.0)
        //     .process_always(true)
        //     .process_in_physics(true)
        //     .done()
        //     .unwrap();
        // t.signals()
        //     .timeout()
        //     .connect_other(&self.to_gd(), Self::setup);

        self.base_mut().call_deferred("setup", &[]);
    }

    fn physics_process(&mut self, _delta: f32) {
        if self.nav_agent.is_navigation_finished() {
            println!("Finished naving");
            return;
        }

        let cur_pos = self.base().get_global_position();
        let next = self.nav_agent.get_next_path_position();
        let vel = cur_pos.direction_to(next) * self.speed;
        self.base_mut().set_velocity(vel);
        self.base_mut().move_and_slide();
    }
}

#[godot_api]
impl NavEnemy {
    async fn setup(&mut self) -> godot::task::SignalFuture<()> {
        let tree = self.base().get_tree().unwrap();

        // let p = tree.signals().physics_frame().to_future().await;

        // godot::task::spawn(async move {
        //     let x: () = Signal::from_object_signal(&tree, "physics_frame")
        //         .to_future()
        //         .await;
        // });

        self.nav_agent.set_target_position(self.target_pos);

        self.set_movement_target(self.target_pos);
        tree.signals().physics_frame().to_future()
    }

    fn set_movement_target(&mut self, target: Vector2) {
        self.nav_agent.set_target_position(target);
    }
}
