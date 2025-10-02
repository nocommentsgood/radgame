use godot::{
    builtin::Vector2,
    classes::{Area2D, CharacterBody2D, ICharacterBody2D, Node},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};
use statig::prelude::StateMachine;

use super::enemy_state_machine::{EnemyEvent, State};
use crate::entities::{
    damage::{AttackData, Damage, DamageType},
    enemies::{
        enemy_state_machine::EnemySMType,
        has_enemy_sensors::EnemySensors,
        physics::{Movement, PhysicsFrameData, Speeds},
        time::{EnemyTimers, Timers},
    },
    ent_graphics::EntGraphics,
    movements::Direction,
};

enum EnemyType {
    EnemyBodyActor,
}

#[derive(Clone)]
struct EnemyContext {
    movement: Movement,
    graphics: EntGraphics,
    sensors: EnemySensors,
    timers: Timers,
    sm: EnemySMType,
}

impl EnemyContext {
    pub fn new(
        node: &Gd<Node>,
        speeds: Speeds,
        left_patrol_target: Vector2,
        right_patrol_target: Vector2,
    ) -> Self {
        Self {
            movement: Movement::new(speeds, left_patrol_target, right_patrol_target),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::new(node),
            timers: Timers::new(node),
            sm: EnemySMType::Basic(StateMachine::default()),
        }
    }

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
            movement: Movement::new(speeds, left_patrol_target, right_patrol_target),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::new(node),
            timers: Timers::new(node),
            sm: EnemySMType::Basic(StateMachine::default()),
        };
        this.sm.inner().init();

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

    fn physics_process(&mut self, frame: PhysicsFrameData) {
        if self.sensors.is_any_raycast_colliding() {
            self.sm.handle(&EnemyEvent::RayCastNotColliding);
        }

        if !frame.on_floor {
            self.sm.handle(&EnemyEvent::FailedFloorCheck);
        }

        match self.sm.state() {
            State::ChasePlayer {} => {
                self.sensors.player_detection.track_player_position();
                if self
                    .sensors
                    .player_detection
                    .attack_area
                    .has_overlapping_areas()
                    && self.timers.attack.get_time_left() == 0.0
                {
                    self.timers.attack.start();
                    self.sm.handle(&EnemyEvent::InAttackRange);
                }
            }
            State::Falling {} => {
                if frame.on_floor {
                    self.sm.handle(&EnemyEvent::OnFloor);
                }
            }
            _ => (),
        }

        self.movement.update(&frame);
        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity()),
        );
    }
}

/// Basic enemy type with a base of type CharacterBody2D.
#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct EnemyBodyActor {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::from_base_fn(|base|
        EnemyContext::new_and_init(
            base,
            Speeds::new(100.0, 150.0),
            Vector2::default(),
            Vector2::default(),
            EnemyType::EnemyBodyActor,
        )))]
    base: OnReady<EnemyContext>,
    body: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for EnemyBodyActor {
    fn ready(&mut self) {
        self.base
            .movement
            .set_patrol_targets(self.left_target, self.right_target);

        self.base.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.base.sensors.hit_reg.hurtbox.clone(),
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });
    }

    fn physics_process(&mut self, delta: f32) {
        let frame = self.new_phy_frame(delta);
        self.base.physics_process(frame);
        let v = self.base.movement.velocity();
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();
    }
}

impl EnemyBodyActor {
    fn new_phy_frame(&self, delta: f32) -> PhysicsFrameData {
        PhysicsFrameData::new(
            self.base.sm.state().clone(),
            self.base().is_on_floor(),
            self.base().get_global_position(),
            self.base.sensors.player_position(),
            delta,
        )
    }

    fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.base.sm.handle(&EnemyEvent::FoundPlayer);
    }

    fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.base.sm.handle(&EnemyEvent::LostPlayer);
    }

    fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        self.base.sm.handle(&EnemyEvent::InAttackRange);
    }

    fn on_idle_timeout(&mut self) {
        if self.base.sm.state() == (&State::Idle {}) {
            self.base.movement.patrol();

            self.base
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
            self.base.timers.patrol.start();
        }
    }

    fn on_patrol_timeout(&mut self) {
        if self.base.sm.state() == (&State::Patrol {}) {
            self.base
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Patrol));
            self.base.timers.idle.start();
        }
    }
}
