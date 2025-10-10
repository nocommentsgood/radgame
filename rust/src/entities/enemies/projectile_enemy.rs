use super::{enemy_state_machine::State, projectile::Projectile};
use crate::entities::{
    damage::{AttackData, Damage, DamageType},
    enemies::{
        enemy_context::EnemyContext,
        enemy_state_machine::{EnemyEvent, EnemySMType},
        physics::{MovementStrategy, Speeds},
        time::{EnemyTimers, Timers},
    },
};
use godot::{
    builtin::Vector2,
    classes::{Area2D, INode2D, Node2D, PackedScene},
    obj::{Base, Gd, OnReady, WithBaseField},
    prelude::{GodotClass, godot_api},
    tools::load,
};
use statig::prelude::StateMachine;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct NewProjectileEnemy {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::manual())]
    projectile_scene: OnReady<Gd<PackedScene>>,

    #[init(val = OnReady::manual())]
    inst: OnReady<Gd<Projectile>>,

    #[init(val = OnReady::manual())]
    ctx: OnReady<EnemyContext>,
    node: Base<Node2D>,
}

#[godot_api]
impl INode2D for NewProjectileEnemy {
    fn ready(&mut self) {
        self.projectile_scene
            .init(load("res://world/projectile.tscn"));

        self.inst
            .init(self.projectile_scene.instantiate_as::<Projectile>());

        let this = self.to_gd();
        let ctx = EnemyContext::default_new(
            &this.clone().upcast(),
            Speeds::new(150.0, 175.0),
            self.left_target,
            self.right_target,
            Timers::default_new(&self.to_gd().upcast()),
            EnemySMType::Basic(StateMachine::default()),
        );
        self.ctx.init(ctx);
        self.ctx.timers.connect_signals(
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
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_chain_timeout()
            },
            {
                let mut this = this.clone();
                move || this.bind_mut().on_attack_anim_timeout()
            },
        );

        self.ctx.sensors.connect_signals(
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

        self.ctx.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });
        self.ctx.timers.idle.start();
    }

    fn process(&mut self, delta: f32) {
        match self.ctx.sm.state() {
            State::ChasePlayer {}
                if self.ctx.sensors.player_detection.attack_area_overlapping()
                    && self.ctx.timers.attack.get_time_left() == 0.0 =>
            {
                self.ctx.sm.handle(&EnemyEvent::InAttackRange);
            }
            State::Attack {} if self.ctx.timers.attack.get_time_left() == 0.0 => {
                self.shoot_projectile();
                self.ctx.timers.attack.start();
                self.ctx.timers.attack_anim.start();
            }
            State::Attack2 {} if self.ctx.timers.attack.get_time_left() == 0.0 => {
                self.shoot_projectile();
                self.ctx.timers.attack_chain.start();
                self.ctx.timers.attack.start();
            }
            _ => (),
        }

        let this = self.to_gd();
        self.ctx.update_movement(
            &mut MovementStrategy::ManualSetPosition(this.upcast()),
            delta,
        );
        self.ctx.update_graphics();
    }
}

#[godot_api]
impl NewProjectileEnemy {
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
            self.ctx
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
            self.ctx.timers.patrol.start();
            self.ctx.movement.patrol();
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

    pub fn on_attack_timeout(&mut self) {
        self.ctx
            .sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Attack));
    }

    fn on_attack_chain_timeout(&mut self) {
        self.shoot_projectile();
        self.ctx
            .sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::AttackChain));
    }

    fn on_attack_anim_timeout(&mut self) {
        self.ctx
            .sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::AttackAnimation));
    }

    pub fn shoot_projectile(&mut self) {
        if let Some(player_pos) = self.ctx.sensors.player_detection.player_position() {
            let mut inst = self.projectile_scene.instantiate_as::<Projectile>();
            let target = self
                .base()
                .get_global_position()
                .direction_to(player_pos)
                .normalized_or_zero();
            let pos = self.base().get_global_position();

            inst.set_global_position(pos);
            inst.bind_mut().velocity = target * 500.0;
            self.base_mut().add_sibling(&inst);
        }
    }
}
