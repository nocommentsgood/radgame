use godot::{
    classes::{Area2D, CharacterBody2D, ICharacterBody2D},
    obj::WithBaseField,
    prelude::*,
};

use super::enemy_state_machine::{EnemyEvent, EnemyStateMachine, State};
use crate::entities::{
    damage::{AttackData, Damage, DamageType},
    enemies::{
        has_enemy_sensors::EnemySensors,
        physics::{self, FrameData, Movement, PatrolComp},
        time::{EnemyTimers, Timers},
    },
    ent_graphics::EntGraphics,
    movements::Direction,
};

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct ModularTestEnemy {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::manual())]
    movement: OnReady<Movement>,

    #[init(val = OnReady::from_base_fn(EntGraphics::new))]
    graphics: OnReady<EntGraphics>,

    #[init(val = OnReady::from_base_fn(EnemySensors::new))]
    sensors: OnReady<EnemySensors>,

    #[init(val = OnReady::from_base_fn(Timers::new))]
    timers: OnReady<Timers>,

    patrol: PatrolComp,
    sm: statig::blocking::StateMachine<EnemyStateMachine>,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for ModularTestEnemy {
    fn ready(&mut self) {
        self.patrol.right_target = self.right_target;
        self.patrol.left_target = self.left_target;
        self.movement.init(physics::Movement::new(
            physics::Speeds::new(100.0, 150.0),
            Vector2::default(),
        ));
        self.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.sensors.hit_reg.hurtbox.clone(),
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });

        self.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_aggro_area_entered);
        self.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_exited()
            .connect_other(&self.to_gd(), Self::on_aggro_area_exited);
        self.sensors
            .player_detection
            .attack_area
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_attack_area_entered);

        self.timers
            .idle
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_idle_timeout);
        self.timers
            .patrol
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_patrol_timeout);
        self.timers
            .attack
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_attack_timeout);

        self.timers.idle.start();
    }

    fn physics_process(&mut self, delta: f32) {
        if self.sensors.is_any_raycast_colliding() {
            self.sm.handle(&EnemyEvent::RayCastNotColliding);
        }

        if !self.base().is_on_floor() {
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
                if self.base().is_on_floor() {
                    self.sm.handle(&EnemyEvent::OnFloor);
                }
            }
            _ => (),
        }

        let frame = FrameData::new(
            self.sm.state(),
            self.base().is_on_floor(),
            self.base().get_global_position(),
            self.sensors.player_position(),
            &self.patrol,
            delta,
        );

        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity),
        );
        self.movement.update(&frame);
        let v = self.movement.velocity;
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();

        // dbg!(&self.sm.state());
    }
}

impl ModularTestEnemy {
    fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&EnemyEvent::FoundPlayer);
    }

    fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&EnemyEvent::LostPlayer);
    }

    fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        if self.timers.attack.get_time_left() == 0.0 {
            self.timers.attack.start();
            self.sm.handle(&EnemyEvent::InAttackRange);
        }
    }

    fn on_idle_timeout(&mut self) {
        let frame = FrameData::new(
            self.sm.state(),
            self.base().is_on_floor(),
            self.base().get_global_position(),
            self.sensors.player_position(),
            &self.patrol,
            self.base().get_physics_process_delta_time() as f32,
        );
        if self.sm.state() == (&State::Idle {}) {
            self.timers.patrol.start();
            self.movement.update_patrol_target(&frame);

            self.sm.handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
        }
    }

    fn on_patrol_timeout(&mut self) {
        if self.sm.state() == (&State::Patrol {}) {
            self.movement.stop();
            self.timers.idle.start();
            self.sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Patrol));
        }
    }

    fn on_attack_timeout(&mut self) {
        self.sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Attack));
    }
}
