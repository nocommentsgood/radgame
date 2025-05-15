use crate::{
    classes::{
        characters::{character_stats::CharacterStats, main_character::MainCharacter},
        components::{speed_component::SpeedComponent, timer_component::EnemyTimers},
        enemies,
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

use crate::classes::components::timer_component::Timer;

use super::patrol_component::PatrolComponent;

const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    shoot_cooldown: Timer,
    patrol_comp: PatrolComponent,
    speeds: SpeedComponent,
    direction: PlatformerDirection,
    stats: CharacterStats,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    timers: EnemyTimers,
    base: Base<Node2D>,
    #[init(load = "res://projectile.tscn")]
    projectile_scene: OnReady<Gd<PackedScene>>,
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
        self.speeds = SpeedComponent::new(40.0, 40.0, 80.0);
        self.patrol_comp = PatrolComponent::new(138.0, 0.0, -138.0, 0.0);
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();
        self.timers = EnemyTimers::new(1.8, 2.0, 1.0, 2.0, 2.0);
        self.stats.health = 20;
        self.shoot_cooldown = Timer::new(2.0);
    }

    fn process(&mut self, _delta: f64) {
        match self.state.state() {
            enemy_state_machine::State::Idle {} => self.idle(),
            enemy_state_machine::State::Attack2 { player } => {
                self.chain_attack(player.clone());
            }
            enemy_state_machine::State::ChasePlayer { player } => self.chase_player(player.clone()),
            enemy_state_machine::State::Patrol {} => self.patrol(),
            enemy_state_machine::State::Falling {} => (),
            enemy_state_machine::State::Attack { player } => {
                self.attack(player.clone());
            }
        }
        self.update_timers();
    }
}

#[godot_api]
impl ProjectileEnemy {
    fn shoot_projectile(&mut self, target: Vector2) {
        let position = self.base().get_global_position();
        let mut bullet = self
            .projectile_scene
            .instantiate_as::<enemies::projectile::Projectile>();
        let target = position.direction_to(target).normalized_or_zero();
        bullet.bind_mut().velocity = target * BULLET_SPEED;
        self.base_mut()
            .call_deferred("add_child", &[bullet.to_variant()]);
    }

    fn attack(&mut self, player: Gd<MainCharacter>) {
        let target = player.get_global_position();
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_process_delta_time() as f32;
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value == attack_cooldown.initial_value() {
            self.shoot_projectile(target);
            self.timers.attack_cooldown.value -= delta;
        }
        self.timers.attack_cooldown.value -= delta;
        self.timers.attack_animation.value -= delta;

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_process_delta_time() as f32;
        self.timers.chain_attack.value -= delta;

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn update_timers(&mut self) {
        let delta = self.base().get_process_delta_time() as f32;

        // Update attack cooldown timer
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value < attack_cooldown.initial_value() && attack_cooldown.value > 0.0 {
            self.timers.attack_cooldown.value -= delta;
        } else if attack_cooldown.value <= 0.0 {
            self.timers.attack_cooldown.reset();
        }
    }
}

#[godot_dyn]
impl CharacterResources for ProjectileEnemy {
    fn get_health(&self) -> u32 {
        self.stats.health
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.health = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.energy
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.energy = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.mana
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.mana = amount;
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

impl character_components::has_aggro::HasAggroArea for ProjectileEnemy {}

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
    fn timers(&mut self) -> &mut crate::classes::components::timer_component::EnemyTimers {
        &mut self.timers
    }
    fn attack(&mut self, player: Gd<MainCharacter>) {
        let target = player.get_global_position();
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_process_delta_time() as f32;
        let attack_cooldown = self.timers.attack_cooldown.clone();
        self.update_animation();
        if attack_cooldown.value == attack_cooldown.initial_value() {
            self.shoot_projectile(target);
            self.timers.attack_cooldown.value -= delta;
        }
        self.timers.attack_cooldown.value -= delta;
        self.timers.attack_animation.value -= delta;

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_process_delta_time() as f32;
        self.timers.chain_attack.value -= delta;

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
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

    fn patrol_comp(&self) -> PatrolComponent {
        self.patrol_comp.clone()
    }
}
