use std::collections::HashMap;

use super::{
    animatable::Animatable,
    enemy_state_machine::{EnemyStateMachine, State},
    has_enemy_sensors::HasEnemySensors,
    has_state::HasState,
    projectile::Projectile,
};
use crate::entities::{
    damage::{AttackData, Damageable, HasHealth},
    enemies::physics::PatrolComp,
    entity_stats::{Stat, StatVal},
    hit_reg::Hurtbox,
    movements::{Direction, Move, Moveable, MoveableEntity},
    time::EnemyTimer,
};
use godot::{
    classes::{AnimationPlayer, Timer},
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
    patrol_comp: PatrolComp,
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

        self.patrol_comp.left_target = self.left_target;
        self.patrol_comp.right_target = self.right_target;
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

        self.hitbox_mut().bind_mut().damageable_parent = Some(Box::new(self.to_gd()));

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

impl HasState for ProjectileEnemy {
    fn sm_mut(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl HasEnemySensors for ProjectileEnemy {
    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>) {
        self.player_pos = pos;
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

impl Animatable for ProjectileEnemy {
    fn anim_player_mut(&mut self) -> &mut Gd<godot::classes::AnimationPlayer> {
        &mut self.animation_player
    }

    fn get_direction(&self) -> &Direction {
        &self.direction
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = Direction::from_vel(&self.velocity);
        }
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
