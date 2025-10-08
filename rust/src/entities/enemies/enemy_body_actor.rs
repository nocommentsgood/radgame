use godot::{
    builtin::Vector2,
    classes::{Area2D, CharacterBody2D, ICharacterBody2D},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
};

use super::enemy_state_machine::{EnemyEvent, State};
use crate::entities::{
    damage::{AttackData, Damage, DamageType},
    enemies::{
        enemy_context::{EnemyContext, EnemyType, Raycasts},
        physics::Speeds,
        time::EnemyTimers,
    },
};

/// Basic enemy type with a base of type CharacterBody2D.
#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
pub struct EnemyBodyActor {
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
    ctx: OnReady<EnemyContext>,
    body: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for EnemyBodyActor {
    fn ready(&mut self) {
        self.ctx
            .movement
            .set_patrol_targets(self.left_target, self.right_target);

        self.ctx.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            parryable: false,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });
    }

    fn physics_process(&mut self, delta: f32) {
        if !self.base().is_on_floor() {
            self.ctx.sm.handle(&EnemyEvent::FailedFloorCheck);
        }
        if let &State::Falling {} = self.ctx.sm.state()
            && self.base().is_on_floor()
        {
            self.ctx.sm.handle(&EnemyEvent::OnFloor);
        }

        match self.ctx.sm.state() {
            State::Patrol {} | State::ChasePlayer {} if self.ctx.sensors.are_raycasts_failing() => {
                match self.ctx.sensors.which() {
                    Raycasts::Ground(dir) => {
                        self.ctx.sm.handle(&EnemyEvent::RayCastFailed(dir));
                    }
                    Raycasts::Wall(dir) => {
                        self.ctx.sm.handle(&EnemyEvent::RayCastFailed(dir));
                    }
                }
            }
            State::RecoverLeft {} | State::RecoverRight {}
                if !self.ctx.sensors.is_wall_cast_colliding() =>
            {
                self.ctx.sm.handle(&EnemyEvent::WallCastRecovered);
            }
            _ => (),
        }
        let this = self.to_gd();
        self.ctx.update_movement(
            &mut super::physics::MovementStrategy::MoveAndSlide(this.upcast()),
            delta,
        );
        self.ctx.update_graphics();
    }
}

impl EnemyBodyActor {
    pub fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.ctx.sm.handle(&EnemyEvent::FoundPlayer);
    }

    pub fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.ctx.sm.handle(&EnemyEvent::LostPlayer);
        self.ctx.timers.idle.start();
    }

    pub fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        self.ctx.sm.handle(&EnemyEvent::InAttackRange);
    }

    pub fn on_idle_timeout(&mut self) {
        if self.ctx.sm.state() == (&State::Idle {}) {
            self.ctx.movement.patrol();

            self.ctx
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
            self.ctx.timers.patrol.start();
        }
    }

    pub fn on_patrol_timeout(&mut self) {
        if self.ctx.sm.state() == (&State::Patrol {}) {
            self.ctx
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Patrol));
            self.ctx.timers.idle.start();
        }
    }
}
