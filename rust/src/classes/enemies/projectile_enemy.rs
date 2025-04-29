use crate::{
    classes::{
        characters::{character_stats::CharacterStats, main_character::MainCharacter},
        components::timer_component::EnemyTimers,
        enemies,
    },
    components::state_machines::enemy_state_machine::{self, *},
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
    },
};
use godot::{
    classes::{Area2D, Marker2D},
    prelude::*,
};

use crate::classes::components::timer_component::Timer;

use super::patrol_component::PatrolComponent;

const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    shoot_cooldown: Timer,
    stats: CharacterStats,
    #[init(val = 80.0)]
    aggro_speed: real,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    timers: EnemyTimers,
    base: Base<Node2D>,

    #[init(node = "EastMarker")]
    east_marker: OnReady<Gd<Marker2D>>,

    #[init(node = "WestMarker")]
    west_marker: OnReady<Gd<Marker2D>>,

    #[init(load = "res://projectile.tscn")]
    projectile_scene: OnReady<Gd<PackedScene>>,

    #[init(node = "MovementLimit")]
    movement_limit_area: OnReady<Gd<Area2D>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
        self.timers = EnemyTimers::new(1.8, 2.0, 1.0, 2.0, 2.0);
        self.stats.health = 20;
        self.connect_signals();
        let spawn_position = self.base().get_global_position();
        self.shoot_cooldown = Timer::new(2.0);
    }

    fn process(&mut self, _delta: f64) {
        // dbg!(&self.state.state());
        // dbg!(&self.velocity);
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
    }
}

#[godot_api]
impl ProjectileEnemy {
    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let self_gd = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(self_gd);
        damaging.dyn_bind().do_damage(damageable);
    }

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

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            if let Some(player) = area.get_parent() {
                if let Ok(player) = player.try_cast::<MainCharacter>() {
                    self.state
                        .handle(&enemy_state_machine::EnemyEvent::FoundPlayer {
                            player: player.clone(),
                        })
                }
            }
        }
    }

    fn on_aggro_area_exited(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            self.state
                .handle(&enemy_state_machine::EnemyEvent::LostPlayer);
        }
    }

    fn connect_signals(&mut self) {
        // Connect player enters aggro area
        let mut aggro = self.base().get_node_as::<Area2D>("AggroArea");
        let mut this = self.to_gd();
        aggro
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_aggro_area_entered(area));

        // Connect player exits aggro area
        let mut this = self.to_gd();
        aggro
            .signals()
            .area_exited()
            .connect(move |area| this.bind_mut().on_aggro_area_exited(area));

        // Connect hitbox
        let mut this = self.to_gd();
        let mut hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));
    }

    fn idle(&mut self) {
        let time = self.timers.idle.value;
        let delta = self.base().get_physics_process_delta_time();
        // self.update_direction();
        // self.update_animation();
        self.velocity = Vector2::ZERO;
        self.timers.idle.value -= delta;

        if time <= 0.0 {
            self.timers.idle.reset();
            self.velocity = self.furthest_patrol_marker_distance();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed)
        }
    }

    fn attack(&mut self, player: Gd<MainCharacter>) {
        let target = player.get_position();
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_physics_process_delta_time();
        self.shoot_projectile(target);
        self.timers.attack_animation.value -= delta;
        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self, player: Gd<MainCharacter>) {
        let target = player.get_position();
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_physics_process_delta_time();
        // let velocity = self.velocity;
        // let speed = self.attack_speed;
        self.timers.chain_attack.value -= delta;
        self.shoot_projectile(target);
        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self.base().get_node_as::<Area2D>("AttackArea");
        let delta = self.base().get_process_delta_time();
        let player_position = player.get_position();
        let position = self.base().get_position();
        let velocity = Vector2::new(
            position
                .direction_to(player_position)
                .normalized_or_zero()
                .x,
            0.0,
        ) * 80.0;
        self.velocity = velocity;

        self.base_mut()
            .set_position(velocity + position * delta as f32);

        if attack_range.has_overlapping_bodies()
            && self.timers.attack_cooldown.value == self.timers.attack_cooldown.initial_value()
        {
            self.state
                .handle(&enemy_state_machine::EnemyEvent::InAttackRange);
            self.timers.attack_cooldown.value -= delta;
        }
    }
    fn furthest_patrol_marker_distance(&self) -> Vector2 {
        let left = &*self.west_marker;
        let right = &*self.east_marker;
        let left_distance = self
            .base()
            .get_position()
            .distance_to(left.get_global_position());
        let right_distance = self
            .base()
            .get_position()
            .distance_to(right.get_global_position());

        let target = if left_distance <= right_distance {
            left
        } else {
            right
        };

        let velocity = self
            .base()
            .get_position()
            .direction_to(target.get_global_position())
            .normalized_or_zero();
        velocity
    }

    fn patrol(&mut self) {
        let time = self.timers.patrol.value;
        let delta = self.base().get_process_delta_time();
        let velocity = self.velocity * 40.0;
        let position = self.base().get_position();
        println!("velocity: {}", velocity);
        println!("position: {position}");

        self.base_mut()
            .set_position(velocity + position * delta as f32);

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
