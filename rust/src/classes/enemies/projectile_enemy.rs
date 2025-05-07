use crate::{
    classes::{
        characters::{character_stats::CharacterStats, main_character::MainCharacter},
        components::timer_component::EnemyTimers,
        enemies,
    },
    components::state_machines::enemy_state_machine::{self, *},
    traits::components::character_components::{
        self, character_resources::CharacterResources, damageable::Damageable,
        moveable::MoveableEntity, *,
    },
};
use godot::{classes::Area2D, prelude::*};
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
    stats: CharacterStats,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    timers: EnemyTimers,
    base: Base<Node2D>,
    #[init(val = 80.0)]
    aggro_speed: real,
    #[init(val = 40.0)]
    patrol_speed: real,
    #[init(load = "res://projectile.tscn")]
    projectile_scene: OnReady<Gd<PackedScene>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
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
            enemy_state_machine::State::Falling {} => println!("FALLING"),
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

    fn idle(&mut self) {
        let time = self.timers.idle.value;
        let delta = self.base().get_process_delta_time();
        self.velocity = Vector2::ZERO;
        self.timers.idle.value -= delta;

        if time <= 0.0 {
            self.timers.idle.reset();
            self.velocity = self
                .patrol_comp
                .get_furthest_distance(self.base().get_global_position());
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed)
        }
    }

    fn attack(&mut self, player: Gd<MainCharacter>) {
        let target = player.get_global_position();
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_process_delta_time();
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
        let delta = self.base().get_process_delta_time();
        self.timers.chain_attack.value -= delta;

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self.base().get_node_as::<Area2D>("EnemySensors/AttackArea");
        let player_position = player.get_global_position();
        let position = self.base().get_global_position();
        let velocity =
            Vector2::new(position.direction_to(player_position).x, 0.0) * self.aggro_speed;
        self.velocity = velocity;
        self.move_to(velocity);

        if attack_range.has_overlapping_bodies()
            && self.timers.attack_cooldown.value == self.timers.attack_cooldown.initial_value()
        {
            self.state
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
        }
    }

    fn update_timers(&mut self) {
        let delta = self.base().get_process_delta_time();

        // Update attack cooldown timer
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value < attack_cooldown.initial_value() && attack_cooldown.value > 0.0 {
            self.timers.attack_cooldown.value -= delta;
        } else if attack_cooldown.value <= 0.0 {
            self.timers.attack_cooldown.reset();
        }
    }

    fn patrol(&mut self) {
        let time = self.timers.patrol.value;
        let delta = self.base().get_process_delta_time();
        let velocity = self.velocity * self.patrol_speed;

        self.move_to(velocity);
        self.timers.patrol.value -= delta;
        if time <= 0.0 {
            self.timers.patrol.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
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
    fn get_mut_sm(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn get_sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl character_components::has_aggro::HasAggroArea for ProjectileEnemy {}

impl character_components::has_hitbox::HasEnemyHitbox for ProjectileEnemy {}

impl character_components::moveable::MoveableEntity for ProjectileEnemy {}
