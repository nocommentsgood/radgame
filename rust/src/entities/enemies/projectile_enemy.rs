use std::collections::HashMap;

use super::{
    enemy_state_machine::{EnemyStateMachine, State},
    projectile::Projectile,
};
use crate::entities::{
    damage::{AttackData, Damage, DamageType, Damageable, HasHealth},
    enemies::{
        enemy_context::{EnemyContext, EnemyType},
        enemy_state_machine::EnemyEvent,
        physics::{MovementStrategy, Speeds},
        time::EnemyTimers,
    },
    entity_stats::{Stat, StatVal},
    hit_reg::Hurtbox,
    movements::{Direction, Move, Moveable, MoveableEntity},
    time::EnemyTimer,
};
use godot::{
    classes::{AnimationPlayer, Area2D, Timer},
    prelude::*,
};

type ET = EnemyTimer;
const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    chain_attack_count: u32,
    // speeds: EnemySpeeds,
    direction: Direction,
    stats: HashMap<Stat, StatVal>,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    #[init(val = HashMap::with_capacity(4))]
    timers: HashMap<EnemyTimer, Gd<Timer>>,
    player_pos: Option<Vector2>,
    base: Base<Node2D>,

    #[init(val = OnReady::manual())]
    projectile_scene: OnReady<Gd<PackedScene>>,

    #[export]
    #[export_subgroup(name = "PatrolComponent")]
    left_target: Vector2,

    #[export]
    #[export_subgroup(name = "PatrolComponent")]
    right_target: Vector2,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<godot::classes::NavigationAgent2D>>,

    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
        self.nav_agent
            .signals()
            .velocity_computed()
            .connect_other(&self.to_gd(), Self::on_velocity_computed);

        self.projectile_scene.init(load("uid://bh5oo6002wig6"));

        self.timers
            .insert(ET::AttackAnimation, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::AttackCooldown, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::AttackChainCooldown, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::Idle, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::Patrol, godot::classes::Timer::new_alloc());

        self.timers
            .get_mut(&ET::AttackAnimation)
            .unwrap()
            .set_wait_time(0.7);
        self.timers
            .get_mut(&ET::AttackCooldown)
            .unwrap()
            .set_wait_time(5.0);
        self.timers
            .get_mut(&ET::AttackChainCooldown)
            .unwrap()
            .set_wait_time(0.4);
        self.timers.get_mut(&ET::Idle).unwrap().set_wait_time(1.5);

        let mut ts = self.timers.clone();
        ts.values_mut().for_each(|timer| {
            timer.set_one_shot(true);
            self.base_mut().add_child(&timer.clone());
        });

        // self.connect_signals();

        self.stats.insert(Stat::Health, StatVal::new(20));
        self.stats.insert(Stat::MaxHealth, StatVal::new(20));
        self.stats.insert(Stat::AttackDamage, StatVal::new(10));

        // self.idle();
        self.animation_player.play_ex().name("idle_east").done();
    }

    fn process(&mut self, _delta: f64) {
        // match self.state.state() {
        //     State::ChasePlayer {} => {
        //         if let Some(p) = self.get_player_pos() {
        //             self.nav_agent.set_target_position(p);
        //         }
        //         self.chase_player()
        //     }
        //     State::Patrol {} => self.patrol(),
        //     _ => (),
        // }

        // dbg!(self.state.state());
    }
}

#[godot_api]
impl ProjectileEnemy {
    fn on_velocity_computed(&mut self, safe_vel: Vector2) {
        if self.state.state().as_discriminant() == (State::ChasePlayer {}).as_discriminant()
            && safe_vel.length() > 0.0
        {
            self.set_velocity(safe_vel);
            self.slide();
        }
    }

    fn shoot_projectile(&mut self, target: Vector2) {
        let position = self.base().get_global_position();
        let target = position.direction_to(target).normalized_or_zero();
        let mut inst = self.projectile_scene.instantiate_as::<Projectile>();
        inst.set_global_position(position);
        let mut hurtbox = inst.get_node_as::<Hurtbox>("Hurtbox");
        inst.bind_mut().velocity = target * BULLET_SPEED;
        hurtbox.set_collision_layer_value(
            crate::utils::collision_layers::CollisionLayers::EnemyHurtbox as i32,
            true,
        );
        hurtbox.set_collision_mask_value(
            crate::utils::collision_layers::CollisionLayers::PlayerHitbox as i32,
            true,
        );
        self.base_mut().add_sibling(&inst);
    }
}

impl Moveable for ProjectileEnemy {
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}

impl MoveableEntity for ProjectileEnemy {}

impl Move for ProjectileEnemy {
    fn slide(&mut self) {
        self.node_slide(false);
    }
}

impl HasHealth for Gd<ProjectileEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().stats.get(&Stat::Health).unwrap().0
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().stats.get_mut(&Stat::Health).unwrap().0 = amount;
    }

    fn on_death(&mut self) {
        self.queue_free();
    }
}

impl Damageable for Gd<ProjectileEnemy> {
    fn handle_attack(&mut self, attack: AttackData) {
        self.take_damage(attack.damage.raw);
    }
}

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

    #[init(val = OnReady::from_base_fn(|base| EnemyContext::new_and_init(base, Speeds::new(150.0, 175.0), Vector2::default(), Vector2::default(), EnemyType::NewProjectileEnemy)))]
    base: OnReady<EnemyContext>,
    node: Base<Node2D>,
}

#[godot_api]
impl INode2D for NewProjectileEnemy {
    fn ready(&mut self) {
        self.projectile_scene
            .init(load("res://world/projectile.tscn"));

        self.inst
            .init(self.projectile_scene.instantiate_as::<Projectile>());
        self.base.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });
        self.base
            .movement
            .set_patrol_targets(self.left_target, self.right_target);

        self.base
            .timers
            .attack_anim
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_attack_anim_timeout);
    }

    fn process(&mut self, delta: f32) {
        // self.base.handle_attack_area();
        self.handle_attack_area();

        if let &State::Attack {} = self.base.sm.state()
            && self.base.timers.attack.get_time_left() == 0.0
        {
            self.shoot_projectile();
            self.base.timers.attack.start();
            self.base.timers.attack_anim.start();
        }
        if let &State::ChasePlayer {} = self.base.sm.state() {
            self.base.sensors.player_detection.track_player_position();
        }

        let this = self.to_gd();
        self.base.update_movement(
            &mut MovementStrategy::ManualSetPosition(this.upcast()),
            delta,
        );
        self.base.update_graphics();
    }
}

#[godot_api]
impl NewProjectileEnemy {
    pub fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.base.sm.handle(&EnemyEvent::FoundPlayer);
    }

    pub fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.base.sm.handle(&EnemyEvent::LostPlayer);
        self.base.timers.idle.start();
    }

    pub fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        if self.base.timers.attack.get_time_left() == 0.0 {
            self.shoot_projectile();
        }
        self.base.sm.handle(&EnemyEvent::InAttackRange);
    }

    pub fn on_idle_timeout(&mut self) {
        if self.base.sm.state() == (&State::Idle {}) {
            self.base
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
            self.base.timers.patrol.start();
            self.base.movement.patrol();
        }
    }

    pub fn on_patrol_timeout(&mut self) {
        if self.base.sm.state() == (&State::Patrol {}) {
            self.base
                .sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Patrol));
            self.base.timers.idle.start();
        }
    }

    pub fn on_attack_timeout(&mut self) {
        self.base
            .sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Attack));
    }

    fn on_attack_anim_timeout(&mut self) {
        self.base
            .sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::AttackAnimation));
    }

    fn handle_attack_area(&mut self) {
        if let State::ChasePlayer {} = self.base.sm.state() {
            self.base.sensors.player_detection.track_player_position();

            if self.base.timers.attack.get_time_left() == 0.0
                && self
                    .base
                    .sensors
                    .player_detection
                    .attack_area
                    .has_overlapping_areas()
            {
                self.base.sm.handle(&EnemyEvent::InAttackRange);
                // self.shoot_projectile();
            }
        }
    }

    fn shoot_projectile(&mut self) {
        if let Some(player_pos) = self.base.sensors.player_position() {
            let mut inst = self.projectile_scene.instantiate_as::<Projectile>();
            let target = self
                .base()
                .get_global_position()
                .direction_to(player_pos)
                .normalized_or_zero();
            let pos = self.base().get_global_position();
            inst.set_global_position(pos);
            inst.bind_mut().velocity = target * 500.0;
            println!("shoot_projectile");
            self.base.timers.attack_anim.start();
            self.base.timers.attack.start();
            self.base_mut().add_sibling(&inst);
        }
    }
}
