use godot::{
    builtin::Vector2,
    classes::{Area2D, Node, Node2D, RayCast2D},
    obj::Gd,
};
use statig::prelude::StateMachine;

use super::enemy_state_machine::{EnemyEvent, State};
use crate::{
    entities::{
        enemies::{
            enemy_body_actor::EnemyBodyActor,
            enemy_state_machine::EnemySMType,
            physics::{Movement, Speeds},
            time::Timers,
        },
        ent_graphics::EntGraphics,
        hit_reg::{HitReg, Hitbox, Hurtbox},
        movements::Direction,
    },
    utils::collision_layers::CollisionLayers,
};

pub enum EnemyType {
    EnemyBodyActor,
}

#[derive(Clone)]
pub struct EnemyContext {
    pub movement: Movement,
    pub graphics: EntGraphics,
    pub sensors: EnemySensors,
    pub timers: Timers,
    pub sm: EnemySMType,
}

impl EnemyContext {
    /// Provides limited default initialization such as connecting timer signal callbacks.
    /// Required methods:
    /// - `on_idle_timeout()` `on_patrol_timeout()`
    /// - `on_aggro_area_entered()` `on_aggro_area_exited()`
    /// - `on_attack_area_entered()`
    pub fn new_and_init(
        node: &Gd<Node>,
        speeds: Speeds,
        left_patrol_target: Vector2,
        right_patrol_target: Vector2,
        ty: EnemyType,
    ) -> Self {
        let mut this = Self {
            movement: Movement::new(
                node.clone().cast::<Node2D>().get_global_position(),
                speeds,
                left_patrol_target,
                right_patrol_target,
            ),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::new(node),
            timers: Timers::new(node),
            sm: EnemySMType::Basic(StateMachine::default()),
        };
        this.sm.inner_mut().init();

        match ty {
            EnemyType::EnemyBodyActor => {
                if let Ok(entity) = node.clone().try_cast::<EnemyBodyActor>() {
                    this.timers
                        .idle
                        .signals()
                        .timeout()
                        .connect_other(&entity, EnemyBodyActor::on_idle_timeout);

                    this.timers
                        .patrol
                        .signals()
                        .timeout()
                        .connect_other(&entity, EnemyBodyActor::on_patrol_timeout);

                    this.sensors
                        .player_detection
                        .aggro_area
                        .signals()
                        .area_entered()
                        .connect_other(&entity, EnemyBodyActor::on_aggro_area_entered);

                    this.sensors
                        .player_detection
                        .aggro_area
                        .signals()
                        .area_exited()
                        .connect_other(&entity, EnemyBodyActor::on_aggro_area_exited);

                    this.sensors
                        .player_detection
                        .attack_area
                        .signals()
                        .area_entered()
                        .connect_other(&entity, EnemyBodyActor::on_attack_area_entered);
                }
            }
        }
        this.timers.idle.start();
        this
    }

    pub fn update_graphics(&mut self) {
        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity()),
        );
    }

    pub fn update_movement(&mut self, strategy: &mut super::physics::MovementStrategy, delta: f32) {
        self.movement.update(
            strategy,
            self.sm.state(),
            self.sensors.player_position(),
            delta,
        );
    }

    pub fn handle_attack_area(&mut self) {
        if let State::ChasePlayer {} = self.sm.state() {
            self.sensors.player_detection.track_player_position();

            if self.timers.attack.get_time_left() == 0.0
                && self
                    .sensors
                    .player_detection
                    .attack_area
                    .has_overlapping_areas()
            {
                // self.timers.attack.start();
                self.sm.handle(&EnemyEvent::InAttackRange);
            }
        }
    }
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

#[derive(Clone)]
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

    pub fn are_raycasts_failing(&self) -> bool {
        self.is_wall_cast_colliding() || !self.is_groundcast_colliding()
    }

    pub fn which(&self) -> Raycasts {
        if self.is_wall_cast_colliding() {
            let dir = self.wall_collision_dir();
            Raycasts::Wall(dir)
        } else if !self.is_groundcast_colliding() {
            let dir = self.groundcast_no_collision_dir();
            Raycasts::Ground(dir)
        } else {
            unreachable!()
        }
    }

    pub fn is_wall_cast_colliding(&self) -> bool {
        self.left_wall_cast.is_colliding() || self.right_wall_cast.is_colliding()
    }

    fn is_groundcast_colliding(&self) -> bool {
        self.left_ground_cast.is_colliding() || self.right_wall_cast.is_colliding()
    }

    pub fn wall_collision_dir(&self) -> Direction {
        if self.is_left_wallcast_colliding() {
            Direction::Left
        } else if self.is_right_wallcast_colliding() {
            Direction::Right
        } else {
            unreachable!(
                "is_wall_cast_colliding returned true but wall_collision_dir couldn't return a direction"
            );
        }
    }

    /// The direction of the groundcast that is not colliding.
    /// Groundcast should always be colliding, else the entity is about to fall.
    pub fn groundcast_no_collision_dir(&self) -> Direction {
        if !self.left_ground_cast.is_colliding() {
            Direction::Left
        } else if !self.right_ground_cast.is_colliding() {
            Direction::Right
        } else {
            unreachable!()
        }
    }

    fn is_left_wallcast_colliding(&self) -> bool {
        self.left_wall_cast.is_colliding()
    }

    pub fn is_right_wallcast_colliding(&self) -> bool {
        self.right_wall_cast.is_colliding()
    }

    pub fn player_position(&self) -> Option<Vector2> {
        self.player_detection.player_position
    }
}

pub enum Raycasts {
    Ground(Direction),
    Wall(Direction),
}
