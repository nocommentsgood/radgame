use godot::{
    classes::{CollisionShape2D, IStaticBody2D, StaticBody2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(init, base = StaticBody2D)]
pub struct ClosingDoor {
    /// The final open position of the door.
    #[export(range = (0.0, -1.0, or_less))]
    open_position: Vector2,

    /// The final closed position of the door.
    #[export(range = (0.0, 1.0, or_greater))]
    closed_position: Vector2,

    is_closed: bool,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for ClosingDoor {
    fn ready(&mut self) {
        self.base_mut().set_process(false);
        let mut shape = self
            .base()
            .get_node_as::<CollisionShape2D>("CollisionShape2D");
        shape.set_disabled(true);
    }

    fn process(&mut self, delta: f32) {
        if !self.is_closed {
            if self.base().get_position().y > self.get_closed_position().y {
                let position = self.base().get_position();
                let x = self.get_closed_position().x;
                self.base_mut()
                    .set_position(Vector2::new(x, position.y + Vector2::UP.y * 20.0 * delta));
            } else {
                println!("Closed!\n {}", self.base().get_position());
                self.is_closed = true;
                self.base_mut().set_process(false);
            }
        }

        if self.is_closed {
            if self.base().get_position().y < self.get_open_position().y {
                let position = self.base().get_position();
                let x = self.get_open_position().x;
                self.base_mut()
                    .set_position(Vector2::new(x, position.y + Vector2::UP.y * 20.0 * delta));
            } else {
                println!("Open!\n {}", self.base().get_position());
                self.is_closed = false;
                self.base_mut().set_process(false);
            }
        }
    }
}

#[godot_api]
impl ClosingDoor {
    pub fn close(&mut self) {
        if !self.is_closed {
            let mut shape = self
                .base()
                .get_node_as::<CollisionShape2D>("CollisionShape2D");
            shape.set_deferred("disabled", &false.to_variant());
            self.base_mut().set_process(true);
        }
    }

    pub fn open(&mut self) {
        if self.is_closed {
            let mut shape = self
                .base()
                .get_node_as::<CollisionShape2D>("CollisionShape2D");
            shape.set_deferred("disabled", &true.to_variant());
            self.base_mut().set_process(true);
        }
    }
}
