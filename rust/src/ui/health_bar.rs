use godot::{
    classes::{CanvasLayer, ICanvasLayer, TextureProgressBar},
    prelude::*,
};

use crate::utils::global_data_singleton::GlobalData;

#[derive(GodotClass)]
#[class(init, base=CanvasLayer)]
pub struct HealthBar {
    #[init(node = "Control/CenterContainer/TextureProgressBar")]
    pub health_bar: OnReady<Gd<TextureProgressBar>>,
    base: Base<CanvasLayer>,
}

#[godot_api]
impl ICanvasLayer for HealthBar {
    fn ready(&mut self) {
        if let Some(player) = GlobalData::singleton().bind().player.as_ref() {
            let bind = player.bind();
            let health = bind.resources.borrow().health().amount();
            let max = bind.resources.borrow().health().max();
            self.set_value(health as f64);
            self.health_bar.set_max(max as f64);

            player
                .signals()
                .player_health_changed()
                .connect_other(&self.to_gd(), Self::on_player_health_changed);
        }
    }
}

#[godot_api]
impl HealthBar {
    pub fn on_player_health_changed(&mut self, _previous_health: i64, current_health: i64) {
        self.health_bar.set_value(current_health as f64);
    }

    pub fn set_value(&mut self, val: f64) {
        self.health_bar.set_value(val);
    }
}
