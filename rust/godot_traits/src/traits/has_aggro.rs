use godot::{
    builtin::Vector2,
    classes::Area2D,
    obj::{Gd, Inherits, WithBaseField},
};

use utils::utils::state_machine_events;

use super::has_state::HasState;

pub trait HasAggroArea: HasState
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node>>,
{
    fn set_player_pos(&mut self, player_pos: Vector2);
    fn get_player_pos(&self) -> Vector2;

    fn create_timer(&mut self) {
        let mut timer = self
            .base()
            .upcast_ref()
            .get_tree()
            .unwrap()
            .create_timer(0.2)
            .unwrap();

        let this = self.to_gd();
        timer
            .signals()
            .timeout()
            .connect_obj(&this, Self::check_player_pos);
    }

    fn check_player_pos(&mut self) {
        let aggro_area = self
            .base()
            .upcast_ref()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AggroArea");

        if aggro_area.has_overlapping_areas() {
            let areas = aggro_area.get_overlapping_areas();
            for area in areas.iter_shared() {
                self.set_player_pos(area.get_global_position());
            }
        }
    }

    fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm_mut()
            .handle(&state_machine_events::EnemyEvent::FoundPlayer);
    }

    fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.sm_mut()
            .handle(&state_machine_events::EnemyEvent::LostPlayer);
    }

    fn connect_aggro_area_signal(&mut self) {
        let mut aggro_area = self
            .base()
            .upcast_ref()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AggroArea");
        let mut this = self.to_gd();

        aggro_area
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_aggro_area_entered(area));

        let mut this = self.to_gd();
        aggro_area
            .signals()
            .area_exited()
            .connect(move |area| this.bind_mut().on_aggro_area_exited(area));
    }
}
