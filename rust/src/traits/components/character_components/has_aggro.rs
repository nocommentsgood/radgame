use godot::{
    classes::Area2D,
    obj::{Gd, Inherits, WithBaseField},
};

use crate::{
    classes::characters::main_character::MainCharacter,
    components::state_machines::enemy_state_machine::{self},
};

use super::has_state::HasState;

pub trait HasAggroArea: HasState
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node>>,
{
    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>);

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            if let Some(player) = area.get_parent() {
                if let Ok(player) = player.try_cast::<MainCharacter>() {
                    self.set_player_pos(Some(player.get_global_position()));
                    self.sm_mut()
                        .handle(&enemy_state_machine::EnemyEvent::FoundPlayer {})
                }
            }
        }
    }

    fn track_player(&mut self) {
        let area = self
            .base()
            .upcast_ref()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AggroArea");
        let areas = area.get_overlapping_areas();
        for a in areas.iter_shared() {
            if a.is_in_group("player") {
                let player = a.get_parent().unwrap().cast::<MainCharacter>();
                self.set_player_pos(Some(player.get_global_position()));
            }
        }
    }

    fn on_aggro_area_exited(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            self.set_player_pos(None);
            self.sm_mut()
                .handle(&enemy_state_machine::EnemyEvent::LostPlayer);
        }
    }

    fn connect_aggro_area_signal(&mut self) {
        let aggro_area = self
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
