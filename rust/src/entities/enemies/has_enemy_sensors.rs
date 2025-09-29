use godot::{
    builtin::Vector2,
    classes::{Area2D, Node, Node2D, RayCast2D},
    obj::{Gd, Inherits, WithBaseField},
    register::{IndirectSignalReceiver, SignalReceiver},
};

use super::has_state::HasState;
use crate::{
    entities::hit_reg::{HitReg, Hitbox, Hurtbox},
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

#[derive(Clone)]
pub struct PlayerDetection {
    pub aggro_area: Gd<Area2D>,
    pub attack_area: Gd<Area2D>,
    player_position: Option<Vector2>,
}

impl PlayerDetection {
    pub fn new(mut aggro_area: Gd<Area2D>, mut attack_area: Gd<Area2D>) -> Self {
        aggro_area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);
        attack_area.set_collision_mask_value(CollisionLayers::PlayerHitbox as i32, true);

        let this = Self {
            aggro_area,
            attack_area,
            player_position: Option::None,
        };

        let mut that = this.clone();
        let mut and = this.clone();
        this.aggro_area
            .signals()
            .area_entered()
            .connect(move |area| that.on_aggro_area_entered(area));
        this.aggro_area
            .signals()
            .area_exited()
            .connect(move |area| and.on_aggro_area_exited(area));

        this
    }

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        self.player_position = Some(area.get_global_position());
    }

    fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.player_position = None;
    }

    pub fn track_player_position(&mut self) {
        let areas = self.aggro_area.get_overlapping_areas();
        for area in areas.iter_shared() {
            let player = area
                .get_owner()
                .expect("Aggro_area overlapping areas should be the player's hitbox")
                .cast::<Node2D>();
            self.player_position = Some(player.get_global_position());
        }
    }
}

pub struct EnemySensors {
    pub hit_reg: HitReg,
    pub player_detection: PlayerDetection,
    left_ground_cast: Gd<RayCast2D>,
    right_ground_cast: Gd<RayCast2D>,
    left_wall_cast: Gd<RayCast2D>,
    right_wall_cast: Gd<RayCast2D>,
}

impl EnemySensors {
    pub fn new(base_enemy: &Gd<Node>) -> Self {
        Self {
            hit_reg: HitReg::new(
                base_enemy.get_node_as::<Hitbox>("EnemySensors/Hitbox"),
                base_enemy.get_node_as::<Hurtbox>("EnemySensors/Hurtboxes"),
                base_enemy.try_get_node_as::<RayCast2D>("EnemySensors/LeftWallCast"),
                base_enemy.try_get_node_as::<RayCast2D>("EnemySensors/RightWallCast"),
            ),
            player_detection: PlayerDetection::new(
                base_enemy.get_node_as("EnemySensors/AggroArea"),
                base_enemy.get_node_as("EnemySensors/AttackArea"),
            ),
            left_ground_cast: base_enemy.get_node_as::<RayCast2D>("EnemySensors/LeftGroundCast"),
            right_ground_cast: base_enemy.get_node_as::<RayCast2D>("EnemySensors/RightGroundCast"),
            left_wall_cast: base_enemy.get_node_as::<RayCast2D>("EnemySensors/LeftWallCast"),
            right_wall_cast: base_enemy.get_node_as::<RayCast2D>("EnemySensors/RightWallCast"),
        }
    }

    fn is_wall_cast_colliding(&self) -> bool {
        self.left_wall_cast.is_colliding() || self.right_wall_cast.is_colliding()
    }

    fn is_ground_cast_colliding(&self) -> bool {
        self.left_ground_cast.is_colliding() || self.right_ground_cast.is_colliding()
    }

    pub fn is_any_raycast_colliding(&self) -> bool {
        self.is_wall_cast_colliding() || !self.is_ground_cast_colliding()
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.player_detection.player_position
    }
}
