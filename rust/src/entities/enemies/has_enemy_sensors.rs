use godot::prelude::FromGodot;
use godot::{
    classes::{Area2D, Node, Node2D},
    obj::{DynGd, Gd, Inherits, WithBaseField},
};

use super::enemy_state_machine::EnemyEvent;
use super::has_state::HasState;
use crate::entities::entity_hitbox::EntityHitbox;
use crate::entities::{
    damage::{Damageable, Damaging},
    hurtbox::Hurtbox,
    player::main_character::MainCharacter,
};

pub trait HasEnemySensors: HasState
where
    Self: Inherits<godot::classes::Node2D> + WithBaseField<Base: Inherits<godot::classes::Node>>,
{
    fn sensors(&self) -> Gd<Node2D> {
        self.base()
            .upcast_ref()
            .get_node_as::<Node2D>("EnemySensors")
    }

    fn sensors_mut(&mut self) -> Gd<Node2D> {
        self.base_mut()
            .upcast_mut()
            .get_node_as::<Node2D>("EnemySensors")
    }

    fn aggro_area(&self) -> Gd<Area2D> {
        self.sensors().get_node_as::<Area2D>("AggroArea")
    }

    fn aggro_area_mut(&mut self) -> Gd<Area2D> {
        self.sensors_mut().get_node_as::<Area2D>("AggroArea")
    }

    fn attack_area(&self) -> Gd<Area2D> {
        self.sensors().get_node_as::<Area2D>("AttackArea")
    }

    fn attack_area_mut(&mut self) -> Gd<Area2D> {
        self.sensors_mut().get_node_as::<Area2D>("AttackArea")
    }

    fn hitbox(&self) -> Gd<EntityHitbox> {
        self.sensors().get_node_as::<EntityHitbox>("Hitbox")
    }

    fn hitbox_mut(&mut self) -> Gd<Area2D> {
        self.sensors_mut().get_node_as::<Area2D>("Hitbox")
    }

    fn hurtbox(&self) -> Gd<Hurtbox> {
        self.sensors().get_node_as::<Hurtbox>("Hurtboxes")
    }

    fn hurtbox_mut(&mut self) -> Gd<Hurtbox> {
        self.sensors_mut().get_node_as::<Hurtbox>("Hurtboxes")
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        damaging.dyn_bind().do_damage(damageable);
    }

    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>);

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player")
            && let Some(player) = area.get_parent()
            && let Ok(player) = player.try_cast::<MainCharacter>()
        {
            self.set_player_pos(Some(player.get_global_position()));
            self.sm_mut().handle(&EnemyEvent::FoundPlayer {});
        }
    }

    fn track_player(&mut self) {
        let areas = self.aggro_area().get_overlapping_areas();
        for area in areas.iter_shared() {
            if area.is_in_group("player") {
                let player = area.get_parent().unwrap().cast::<MainCharacter>();
                self.set_player_pos(Some(player.get_global_position()));
            }
        }
    }

    fn on_aggro_area_exited(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            self.set_player_pos(None);
            self.sm_mut().handle(&EnemyEvent::LostPlayer);
        }
    }

    fn connect_signals(&mut self) {
        let this = self.to_gd();
        self.aggro_area_mut()
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_aggro_area_entered);
        self.aggro_area_mut()
            .signals()
            .area_exited()
            .connect_other(&this, Self::on_aggro_area_exited);
        self.hitbox_mut()
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hitbox);
    }
}
