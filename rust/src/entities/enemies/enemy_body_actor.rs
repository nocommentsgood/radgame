use godot::{
    builtin::Vector2,
    classes::{Area2D, CharacterBody2D, ICharacterBody2D},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};
use statig::prelude::StateMachine;

use super::enemy_state_machine as esm;
use crate::entities::{
    enemies::{enemy_context as ctx, physics, time},
    entity,
    movements::Direction,
};

/// Basic enemy type with a base of type CharacterBody2D.
#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
pub struct EnemyBodyActor {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::manual())]
    movement: OnReady<physics::Movement>,
    #[init(val = OnReady::manual())]
    sensors: OnReady<ctx::EnemySensors>,
    #[init(val = OnReady::manual())]
    timers: OnReady<time::Timers>,
    #[init(val = OnReady::manual())]
    sm: OnReady<esm::EnemySMType>,
    #[init(val = OnReady::manual())]
    entity: OnReady<entity::Entity>,
    body: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for EnemyBodyActor {
    fn ready(&mut self) {
        let this = self.to_gd();
        self.entity
            .init(entity::Entity::new(&self.to_gd().upcast()));
        self.movement.init(physics::Movement::new(
            self.base().get_global_position(),
            physics::Speeds::new(150.0, 175.0),
            self.left_target,
            self.right_target,
        ));
        self.sensors
            .init(ctx::EnemySensors::default_new(&self.to_gd().upcast()));
        self.timers
            .init(time::Timers::default_new(&self.to_gd().upcast()));
        self.sm
            .init(esm::EnemySMType::Basic(StateMachine::default()));

        self.timers.connect_signals(
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_patrol_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_idle_timeout()
            },
            || (),
            || (),
        );

        self.sensors.connect_signals(
            |_| (),
            |_| (),
            |_| (),
            |_| (),
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_aggro_area_entered(area)
            },
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_aggro_area_exited(area)
            },
            {
                let mut this = this.clone();
                move |area| this.bind_mut().on_attack_area_entered(area)
            },
            |_| (),
        );
        self.timers.idle.start();

        // self.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
        //     parryable: false,
        //     damage: Damage {
        //         raw: 10,
        //         d_type: DamageType::Physical,
        //     },
        // });
    }

    fn physics_process(&mut self, delta: f32) {
        if !self.base().is_on_floor() {
            self.sm.handle(&esm::EnemyEvent::FailedFloorCheck);
        }
        if let &esm::State::Falling {} = self.sm.state()
            && self.base().is_on_floor()
        {
            self.sm.handle(&esm::EnemyEvent::OnFloor);
        }

        match self.sm.state() {
            esm::State::Patrol {} | esm::State::ChasePlayer {}
                if self.sensors.are_raycasts_failing() =>
            {
                match self.sensors.which() {
                    ctx::Raycasts::Ground(dir) => {
                        self.sm.handle(&esm::EnemyEvent::RayCastFailed(dir));
                    }
                    ctx::Raycasts::Wall(dir) => {
                        self.sm.handle(&esm::EnemyEvent::RayCastFailed(dir));
                    }
                }
            }
            esm::State::RecoverLeft {} | esm::State::RecoverRight {}
                if !self.sensors.is_wall_cast_colliding() =>
            {
                self.sm.handle(&esm::EnemyEvent::WallCastRecovered);
            }
            _ => (),
        }
        let this = self.to_gd();
        self.movement.update(
            &mut physics::MovementStrategy::MoveAndSlide(this.upcast()),
            self.sm.state(),
            self.sensors.player_detection.player_position(),
            delta,
        );
        self.entity.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity()),
        );
    }
}

impl EnemyBodyActor {
    pub fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::FoundPlayer);
    }

    pub fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::LostPlayer);
        self.timers.idle.start();
    }

    pub fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&esm::EnemyEvent::InAttackRange);
    }

    pub fn on_idle_timeout(&mut self) {
        if self.sm.state() == (&esm::State::Idle {}) {
            self.movement.patrol();

            self.sm
                .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Idle));
            self.timers.patrol.start();
        }
    }

    pub fn on_patrol_timeout(&mut self) {
        if self.sm.state() == (&esm::State::Patrol {}) {
            self.sm
                .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Patrol));
            self.timers.idle.start();
        }
    }

    fn on_attack_timeout(&mut self) {
        self.sm
            .handle(&esm::EnemyEvent::TimerElapsed(time::EnemyTimers::Attack));
    }
}
