use godot::{
    builtin::Vector2,
    classes::{Area2D, Node2D, RayCast2D},
    obj::{Gd, Inherits, OnReady, WithBaseField},
};

use super::has_state::HasState;
use crate::{
    entities::{
        hit_reg::{HitReg, Hitbox, Hurtbox},
        player::main_character::MainCharacter,
    },
    utils::collision_layers::CollisionLayers,
};

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

    fn left_wall_cast(&self) -> Gd<RayCast2D> {
        self.sensors().get_node_as::<RayCast2D>("LeftWallCast")
    }

    fn right_wall_cast(&self) -> Gd<RayCast2D> {
        self.sensors().get_node_as::<RayCast2D>("RightWallCast")
    }

    fn left_ground_cast(&self) -> Gd<RayCast2D> {
        self.sensors().get_node_as::<RayCast2D>("LeftGroundCast")
    }

    fn right_ground_cast(&self) -> Gd<RayCast2D> {
        self.sensors().get_node_as::<RayCast2D>("RightGroundCast")
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

    fn hitbox(&self) -> Gd<Hitbox> {
        self.sensors().get_node_as::<Hitbox>("Hitbox")
    }

    fn hitbox_mut(&mut self) -> Gd<Hitbox> {
        self.sensors_mut().get_node_as::<Hitbox>("Hitbox")
    }

    fn hurtbox(&self) -> Gd<Hurtbox> {
        self.sensors().get_node_as::<Hurtbox>("Hurtboxes")
    }

    fn hurtbox_mut(&mut self) -> Gd<Hurtbox> {
        self.sensors_mut().get_node_as::<Hurtbox>("Hurtboxes")
    }

    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>);
}

pub struct PlayerDetection {
    aggro_area: Gd<Area2D>,
    attack_area: Gd<Area2D>,
    player_position: Vector2,
}

impl PlayerDetection {
    pub fn new(mut aggro_area: Gd<Area2D>, mut attack_area: Gd<Area2D>) -> Self {
        aggro_area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        attack_area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        Self {
            aggro_area,
            attack_area,
            player_position: Vector2::default(),
        }
    }

    fn on_player_entered_aggro_area(&mut self, area: Gd<Area2D>) {
        self.player_position = area.get_global_position();
    }

    fn track_player_position(&mut self) {
        let areas = self.aggro_area.get_overlapping_areas();
        for area in areas.iter_shared() {
            let player = area
                .get_owner()
                .expect("Aggro_area overlapping areas should be the player's hitbox")
                .cast::<Node2D>();
            self.player_position = player.get_global_position();
        }
    }
}

pub struct EnemySensors {
    hit_reg: HitReg,
    player_detection: PlayerDetection,
    left_ground_cast: Gd<RayCast2D>,
    right_ground_cast: Gd<RayCast2D>,
}

impl EnemySensors {
    pub fn new(
        hit_reg: HitReg,
        player_detection: PlayerDetection,
        left_ground_cast: Gd<RayCast2D>,
        right_ground_cast: Gd<RayCast2D>,
    ) -> Self {
        Self {
            hit_reg,
            player_detection,
            left_ground_cast,
            right_ground_cast,
        }
    }
}
