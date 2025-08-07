use std::{collections::HashMap, time::Duration};

use super::{
    animatable::Animatable,
    enemy_state_ext::EnemyEntityStateMachineExt,
    enemy_state_machine::{EnemyEvent, EnemyStateMachine, State},
    has_enemy_sensors::HasEnemySensors,
    has_state::HasState,
    patrol_component::PatrolComp,
    projectile::Projectile,
};
use crate::entities::{
    damage::Damageable,
    entity_stats::{EntityResources, StatVal, Stats},
    hurtbox::Hurtbox,
    movements::MoveableEntity,
    movements::{Direction, SpeedComponent},
    time::EnemyTimer,
};
use bevy_time::{Timer, TimerMode};
use godot::{classes::AnimationPlayer, prelude::*};

type ET = EnemyTimer;
const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    shoot_cooldown: Timer,
    speeds: SpeedComponent,
    chain_attack_count: u32,
    direction: Direction,
    stats: HashMap<Stats, StatVal>,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    timers: HashMap<EnemyTimer, bevy_time::Timer>,
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
        let this = self.to_gd();
        self.nav_agent
            .signals()
            .velocity_computed()
            .connect_other(&self.to_gd(), Self::on_velocity_computed);
        self.patrol_comp.left_target = self.left_target;
        self.patrol_comp.right_target = self.right_target;
        self.projectile_scene.init(load("uid://bh5oo6002wig6"));
        self.signals()
            .animation_state_changed()
            .connect_other(&this, Self::on_animation_state_changed);
        self.connect_signals();

        self.timers.insert(
            ET::AttackAnimation,
            Timer::from_seconds(0.7, TimerMode::Repeating),
        );
        self.timers.insert(
            ET::AttackCooldown,
            Timer::from_seconds(0.5, TimerMode::Repeating),
        );
        self.timers.insert(
            ET::AttackChainCooldown,
            Timer::from_seconds(1.5, TimerMode::Repeating),
        );
        self.timers
            .insert(ET::Idle, Timer::from_seconds(3.0, TimerMode::Repeating));
        self.timers
            .insert(ET::Patrol, Timer::from_seconds(2.5, TimerMode::Repeating));

        self.shoot_cooldown = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating);

        self.stats.insert(Stats::Health, StatVal::new(20));
        self.stats.insert(Stats::MaxHealth, StatVal::new(20));
        self.stats.insert(Stats::AttackDamage, StatVal::new(10));
        self.stats.insert(Stats::RunningSpeed, StatVal::new(150));
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(300));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));

        self.speeds = SpeedComponent::new(
            self.stats[&Stats::AttackingSpeed].0,
            self.stats[&Stats::RunningSpeed].0,
            40,
        )
    }

    fn physics_process(&mut self, _delta: f64) {
        let prev_state = self.state.state().as_discriminant();
        match self.state.state() {
            State::Idle {} => self.idle(),
            State::Attack2 {} => {
                self.chain_attack();
            }
            State::ChasePlayer {} => {
                if let Some(p) = self.get_player_pos() {
                    self.nav_agent.set_target_position(p);
                }
                self.chase_player()
            }
            State::Patrol {} => self.patrol(),
            State::Falling {} => (),
            State::Attack {} => self.attack(),
        }

        // Only update animation player when state has changed.
        let new_state = self.state.state().as_discriminant();
        if prev_state != new_state {
            self.signals().animation_state_changed().emit();
        }

        // dbg!(self.state.state());
    }
}

#[godot_api]
impl ProjectileEnemy {
    #[signal]
    fn animation_state_changed();

    fn on_animation_state_changed(&mut self) {
        self.update_animation();
    }

    fn on_velocity_computed(&mut self, safe_vel: Vector2) {
        if self.state.state().as_discriminant() == (State::ChasePlayer {}).as_discriminant() {
            self.move_to(&safe_vel, true);
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
            crate::utils::collision_layers::CollisionLayers::PlayerHurtbox as i32,
            true,
        );
        hurtbox.bind_mut().attack_damage = 20;
        self.base_mut().add_sibling(&inst);
    }
}

#[godot_dyn]
impl EntityResources for ProjectileEnemy {
    fn get_health(&self) -> u32 {
        self.stats.get(&Stats::Health).unwrap().0
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Health).unwrap().0 = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.get(&Stats::Energy).unwrap().0
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Energy).unwrap().0 = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.get(&Stats::Mana).unwrap().0
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Energy).unwrap().0 = amount;
    }
}

#[godot_dyn]
impl Damageable for ProjectileEnemy {
    fn destroy(&mut self) {
        self.base_mut().queue_free();
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

impl MoveableEntity for ProjectileEnemy {}

impl Animatable for ProjectileEnemy {
    fn get_anim_player(&mut self) -> &mut Gd<godot::classes::AnimationPlayer> {
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

impl EnemyEntityStateMachineExt for ProjectileEnemy {
    fn timers(&mut self) -> &mut HashMap<EnemyTimer, Timer> {
        &mut self.timers
    }
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    fn speeds(&self) -> &SpeedComponent {
        &self.speeds
    }

    fn patrol_comp(&self) -> &PatrolComp {
        &self.patrol_comp
    }

    fn attack(&mut self) {
        let delta = Duration::from_secs_f32(self.base().get_process_delta_time() as f32);

        if self.timers.get(&ET::AttackCooldown).unwrap().elapsed_secs() == 0.0
            && let Some(target) = self.get_player_pos()
        {
            self.shoot_projectile(target);
        }

        self.timers
            .get_mut(&ET::AttackCooldown)
            .unwrap()
            .tick(delta);

        if self
            .timers
            .get(&ET::AttackCooldown)
            .unwrap()
            .just_finished()
        {
            self.timers.get_mut(&ET::AttackCooldown).unwrap().reset();
            self.state.handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self) {
        let delta = Duration::from_secs_f32(self.base().get_process_delta_time() as f32);
        // dbg!(delta);
        // dbg!(self.base().get_process_delta_time());
        self.track_player();
        self.shoot_cooldown.tick(delta);

        self.timers
            .get_mut(&ET::AttackChainCooldown)
            .unwrap()
            .tick(delta);

        if self.shoot_cooldown.just_finished() {
            println!("Timer finished");
        }
        if self.shoot_cooldown.just_finished()
            && let Some(target) = self.get_player_pos()
        {
            println!("Timer finished and have target");
            self.shoot_projectile(target);
            self.chain_attack_count += 1;

            self.shoot_cooldown.reset();
        }

        if self.chain_attack_count >= 3
            || self
                .timers
                .get(&ET::AttackChainCooldown)
                .unwrap()
                .finished()
        // if self
        //     .timers
        //     .get(&ET::AttackChainCooldown)
        //     .unwrap()
        //     .finished()
        {
            // self.timers.get_mut(&ET::AttackCooldown).unwrap().reset();
            self.shoot_cooldown.reset();
            self.chain_attack_count = 0;
            self.timers
                .get_mut(&ET::AttackChainCooldown)
                .unwrap()
                .reset();
            self.state.handle(&EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self) {
        let next = self.nav_agent.get_next_path_position();
        let v = self.base().get_global_position().direction_to(next) * self.speeds().aggro;
        self.track_player();
        self.nav_agent.set_velocity(v);

        let ac = &ET::AttackCooldown;
        let delta = Duration::from_secs_f32(self.base().get_process_delta_time() as f32);
        let time = self.timers().get(ac);
        if self.attack_area().has_overlapping_areas()
            && self.timers.get(ac).unwrap().elapsed_secs() == 0.0
        {
            self.timers.get_mut(ac).unwrap().tick(delta);
            self.state.handle(&EnemyEvent::InAttackRange);
        }
    }

    fn get_player_pos(&self) -> Option<Vector2> {
        self.player_pos
    }
}
