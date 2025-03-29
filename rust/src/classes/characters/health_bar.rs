use godot::{
    classes::{CanvasLayer, TextureProgressBar},
    prelude::*,
};

#[derive(GodotClass)]
#[class(init, base=CanvasLayer)]
pub struct HealthBar {
    #[init(node = "Control/CenterContainer/TextureProgressBar")]
    health_bar: OnReady<Gd<TextureProgressBar>>,
    base: Base<CanvasLayer>,
}

#[godot_api]
impl HealthBar {
    fn update_range_value(&mut self, value: f64) {
        self.health_bar.set_value(value);
    }

    pub fn on_player_health_changed(
        &mut self,
        _previous_health: i32,
        current_health: i32,
        _amount: i32,
    ) {
        self.update_range_value(current_health as f64);
    }
}
