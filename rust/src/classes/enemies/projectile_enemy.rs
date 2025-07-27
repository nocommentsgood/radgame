use std::collections::HashMap;

use crate::classes::components::hurtbox::Hurtbox;
use crate::classes::components::timer_component::Timers;
use crate::classes::enemies::projectile::Projectile;
use crate::traits::components::character_components::moveable::MoveableEntity;
use crate::{
    classes::{
        characters::{
            character_stats::{StatVal, Stats},
            main_character::MainCharacter,
        },
        components::speed_component::SpeedComponent,
    },
    components::state_machines::{
        enemy_state_machine::{self, *},
        movements::PlatformerDirection,
    },
    traits::components::character_components::{
        self, animatable::Animatable, character_resources::CharacterResources,
        damageable::Damageable, enemy_state_ext::EnemyEntityStateMachineExt, *,
    },
};
use godot::{classes::AnimationPlayer, prelude::*};
use has_aggro::HasAggroArea;
use has_hitbox::HasEnemyHitbox;

use crate::classes::components::timer_component::{EnemyTimer, Time};

use super::patrol_component::PatrolComp;

type ET = EnemyTimer;
const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    shoot_cooldown: Time,
    speeds: SpeedComponent,
    direction: PlatformerDirection,
    stats: HashMap<Stats, StatVal>,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    timers: Timers,
    patrol_comp: PatrolComp,
    nav_agent_vel: Vector2,
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
        self.projectile_scene.init(load("res://projectile.tscn"));
        self.speeds = SpeedComponent::new(40.0, 40.0, 80.0);
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();

        self.timers.0.push(Time::new(1.8));
        self.timers.0.push(Time::new(2.0));
        self.timers.0.push(Time::new(1.0));
        self.timers.0.push(Time::new(2.0));
        self.timers.0.push(Time::new(2.0));
        self.shoot_cooldown = Time::new(2.0);

        self.stats.insert(Stats::Health, StatVal::new(20));
        self.stats.insert(Stats::MaxHealth, StatVal::new(20));
        self.stats.insert(Stats::AttackDamage, StatVal::new(10));
        self.stats.insert(Stats::RunningSpeed, StatVal::new(150));
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(300));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));
    }

    fn physics_process(&mut self, _delta: f64) {
        match self.state.state() {
            enemy_state_machine::State::Idle {} => self.idle(),
            enemy_state_machine::State::Attack2 {} => {
                self.chain_attack();
            }
            enemy_state_machine::State::ChasePlayer {} => {
                if let Some(p) = self.get_player_pos() {
                    self.nav_agent.set_target_position(p);
                }
                self.chase_player()
            }
            enemy_state_machine::State::Patrol {} => self.patrol(),
            enemy_state_machine::State::Falling {} => (),
            enemy_state_machine::State::Attack {} => {
                self.attack();
            }
        }
        // dbg!(self.state.state());
        self.update_timers();
    }
}

#[godot_api]
impl ProjectileEnemy {
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

    fn update_timers(&mut self) {
        let delta = self.base().get_process_delta_time() as f32;

        // Update attack cooldown timer
        let ac = &ET::AttackCooldown;
        let attack_cooldown = self.timers.get(ac);
        let init = self.timers.get_init(ac);
        if attack_cooldown < init && attack_cooldown > 0.0 {
            self.timers.set(ac, attack_cooldown - delta);
        } else if attack_cooldown <= 0.0 {
            self.timers.reset(ac);
        }
    }
}

#[godot_dyn]
impl CharacterResources for ProjectileEnemy {
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

impl character_components::has_state::HasState for ProjectileEnemy {
    fn sm_mut(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl character_components::has_aggro::HasAggroArea for ProjectileEnemy {
    fn set_player_pos(&mut self, pos: Option<Vector2>) {
        self.player_pos = pos;
    }
}

impl character_components::has_hitbox::HasEnemyHitbox for ProjectileEnemy {}

impl character_components::moveable::MoveableEntity for ProjectileEnemy {}

impl character_components::animatable::Animatable for ProjectileEnemy {
    fn get_anim_player(&self) -> Gd<godot::classes::AnimationPlayer> {
        self.animation_player.clone()
    }

    fn get_direction(&self) -> crate::components::state_machines::movements::PlatformerDirection {
        self.direction.clone()
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }
}

impl character_components::enemy_state_ext::EnemyEntityStateMachineExt for ProjectileEnemy {
    fn timers(&mut self) -> &mut crate::classes::components::timer_component::Timers {
        &mut self.timers
    }
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    fn speeds(&self) -> crate::classes::components::speed_component::SpeedComponent {
        self.speeds.clone()
    }

    fn patrol_comp(&self) -> &PatrolComp {
        &self.patrol_comp
    }

    fn attack(&mut self) {
        let ac = &ET::AttackCooldown;
        let aa = &ET::AttackAnimation;
        let time = self.timers.get(aa);
        let delta = self.base().get_process_delta_time() as f32;
        let attack_cooldown = self.timers.get(ac);
        self.update_animation();

        if attack_cooldown == self.timers.get_init(ac)
            && let Some(target) = self.get_player_pos()
        {
            self.shoot_projectile(target);
            self.timers.set(ac, attack_cooldown - delta);
        }
        self.timers.set(ac, attack_cooldown - delta);
        self.timers.set(aa, time - delta);

        if time <= 0.0 {
            self.timers.reset(aa);
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self) {
        let ac = &ET::AttackChain;
        let time = self.timers.get(ac);
        let delta = self.base().get_process_delta_time() as f32;
        self.timers.set(ac, time - delta);

        if time <= 0.0 {
            self.timers.reset(ac);
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self) {
        let next = self.nav_agent.get_next_path_position();
        let v = self.base().get_global_position().direction_to(next) * self.speeds.aggro;
        self.track_player();
        self.nav_agent.set_velocity(v);

        let ac = &ET::AttackCooldown;
        let attack_range = self
            .base()
            .get_node_as::<godot::classes::Area2D>("EnemySensors/AttackArea");
        let delta = self.base().get_process_delta_time() as f32;
        let time = self.timers().get(ac);
        if attack_range.has_overlapping_areas() && time == self.timers().get_init(ac) {
            self.timers().set(ac, time - delta);
            self.state
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }

    fn get_player_pos(&self) -> Option<Vector2> {
        self.player_pos
    }
}
