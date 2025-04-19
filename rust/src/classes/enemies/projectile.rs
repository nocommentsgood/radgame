use godot::classes;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Projectile {
    pub velocity: Vector2,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Projectile {
    fn process(&mut self, delta: f64) {
        let velocity = self.velocity;
        println!("bullet velocity: {}", velocity);
        self.base_mut().set_position(velocity * delta as f32);
        println!("bullet position: {}", self.base().get_position());
    }
}

#[godot_api]
impl Projectile {
    // fn made_contact(&mut self) -> bool {
    //     let hurtbox = self.base().get_node_as::<Area2D>("Hurtbox");
    // }

    fn on_area_entered(&mut self, area: Gd<classes::Area2D>) {
        if area.is_in_group("player") {}
    }

    fn connect_hitbox(&mut self) {
        let mut hurtbox = self.base().get_node_as::<classes::Area2D>("Hurtbox");
        let mut this = self.to_gd();

        hurtbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered(area));
    }

    fn move_projectile(&mut self) {}
}
