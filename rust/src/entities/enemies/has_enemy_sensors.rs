use godot::{
    classes::{Area2D, Node2D},
    obj::{Gd, Inherits, WithBaseField},
};

use super::has_state::HasState;
use crate::entities::entity_hitbox::EntityHitbox;
use crate::entities::hurtbox::Hurtbox;

#[allow(unused)]
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

    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>);
}
