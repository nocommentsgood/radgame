use godot::{
    classes::{CanvasLayer, TextureProgressBar},
    obj::WithBaseField,
    prelude::*,
};

#[derive(GodotClass)]
#[class(init, base=CanvasLayer)]
pub struct HealthBar {
    base: Base<CanvasLayer>,
}

#[godot_api]
impl HealthBar {
    fn update_range_value(&mut self, value: f64) {
        let mut bar = self
            .base()
            .get_node_as::<TextureProgressBar>("Control/CenterContainer/TextureProgressBar");
        println!("previous range value: {}", bar.get_value());
        bar.set_value(value);
        println!("new range value: {}", bar.get_value());
    }

    #[func]
    fn on_player_health_changed(
        &mut self,
        previous_health: Variant,
        current_health: Variant,
        amount: Variant,
    ) {
        println!("on player damaged");
        let health = current_health
            .try_to::<i32>()
            .expect("health_bar.rs, struct HealthBar{} line 43");
        println!("health is: {}", health);
        self.update_range_value(health as f64);
    }
}
