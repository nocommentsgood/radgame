use godot::{
    classes::{AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, Marker2D},
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    classes::{
        characters::main_character::MainCharacter, components::timer_component::EnemyTimers,
    },
    components::state_machines::{
        enemy_state_machine::{self, *},
        movements::PlatformerDirection,
    },
    traits::components::character_components::{
        animatable::Animatable, character_resources::CharacterResources, damageable::Damageable,
        damaging::Damaging, has_aggro::HasAggroArea, has_hitbox::HasEnemyHitbox,
        moveable::MoveableCharacter,
    },
    utils::*,
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    current_event: enemy_state_machine::EnemyEvent,
    direction: PlatformerDirection,
    timers: EnemyTimers,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    base: Base<CharacterBody2D>,

    #[var]
    #[init(val = 40.0)]
    patrol_speed: real,

    #[var]
    #[init(val = 80.0)]
    agro_speed: real,

    #[init(val = 40.0)]
    attack_speed: real,

    #[var]
    speed: real,

    #[var]
    #[init(node = "LeftPatrolMarker")]
    left_patrol_marker: OnReady<Gd<Marker2D>>,

    #[var]
    #[init(node = "RightPatrolMarker")]
    right_patrol_marker: OnReady<Gd<Marker2D>>,

    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[var]
    #[init(val = 100)]
    health: u32,

    #[var]
    energy: u32,

    #[var]
    mana: u32,

    #[var]
    velocity: Vector2,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();
        self.timers = EnemyTimers::new(1.8, 2.0, 2.7, 2.0, 4.0);
    }

    fn physics_process(&mut self, _delta: f64) {
        self.state.handle(&self.current_event);
        dbg!(&self.current_event);
        self.check_floor();
        // dbg!(&self.state.state());

        match self.state.state() {
            enemy_state_machine::State::Idle {} => self.idle(),
            enemy_state_machine::State::ChasePlayer { player } => self.chase_player(player.clone()),
            enemy_state_machine::State::Patrol {} => self.patrol(),
            enemy_state_machine::State::Attack { player } => self.attack(player.clone()),
            enemy_state_machine::State::Attack2 { player } => self.chain_attack(player.clone()),
            enemy_state_machine::State::Falling {} => self.fall(),
        }

        self.update_timers();
    }
}

#[godot_api]
impl TestEnemy {
    #[signal]
    pub fn test_enemy_died();

    #[signal]
    fn can_attack_player();

    fn check_floor(&mut self) {
        if !self.base().is_on_floor() {
            self.state
                .handle(&enemy_state_machine::EnemyEvent::FailedFloorCheck);
        }
    }

    fn furthest_patrol_marker_distance(&self) -> Vector2 {
        let left = self.get_left_patrol_marker();
        let right = self.get_right_patrol_marker();
        let pos = self.base().get_global_position();
        let left_distance = pos.distance_to(left.get_global_position());
        let right_distance = pos.distance_to(right.get_global_position());

        let target = if left_distance >= right_distance {
            left
        } else {
            right
        };

        let velocity = self
            .base()
            .get_global_position()
            .direction_to(target.get_global_position());
        velocity
    }

    // Leaving this somewhat open ended in case more timers are added later
    fn update_timers(&mut self) {
        let delta = self.base().get_physics_process_delta_time();

        // Update attack cooldown timer
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value < attack_cooldown.initial_value() && attack_cooldown.value > 0.0 {
            self.timers.attack_cooldown.value -= delta;
        } else if attack_cooldown.value <= 0.0 {
            self.timers.attack_cooldown.reset();
        }
    }

    fn attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_physics_process_delta_time();
        let speed = self.attack_speed;
        let velocity = self.velocity;
        self.timers.attack_animation.value -= delta;

        self.slide(&velocity, &speed);
        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        }
    }

    fn chain_attack(&mut self, _player: Gd<MainCharacter>) {
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_physics_process_delta_time();
        let velocity = self.velocity;
        let speed = self.attack_speed;
        self.timers.chain_attack.value -= delta;

        self.slide(&velocity, &speed);
        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        }
    }

    fn patrol(&mut self) {
        let time = self.timers.patrol.value;
        let speed = self.patrol_speed;
        let velocity = self.get_velocity();
        let delta = self.base().get_physics_process_delta_time();
        println!("patrol vel: {velocity}");

        self.update_direction();
        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        self.slide(&velocity, &speed);
        self.timers.patrol.value -= delta;

        if time <= 0.0 {
            self.timers.patrol.reset();
            println!("\n\n\ntimer elapsed from patrol");
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            println!("\n\n\nnone from patrol");
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn idle(&mut self) {
        let time = self.timers.idle.value;
        let delta = self.base().get_physics_process_delta_time();
        let velocity = Vector2::ZERO;
        self.update_direction();
        self.slide(&velocity, &0.0);
        // self.update_animation();
        // self.base_mut().set_velocity(Vector2::ZERO);
        self.timers.idle.value -= delta;

        if time <= 0.0 {
            self.timers.idle.reset();
            self.set_velocity(self.furthest_patrol_marker_distance());
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self.base().get_node_as::<Area2D>("EnemySensors/AttackArea");
        let delta = self.base().get_physics_process_delta_time();
        let speed = self.agro_speed;
        let player_position = player.get_position();
        let velocity = Vector2::new(
            self.base()
                .get_position()
                .direction_to(player_position)
                .normalized_or_zero()
                .x,
            0.0,
        );

        self.velocity = velocity;

        self.update_direction();

        self.slide(&velocity, &speed);

        // self.update_animation();
        // self.base_mut().set_velocity(velocity * speed);
        // self.base_mut().move_and_slide();

        if attack_range.has_overlapping_bodies()
            && self.timers.attack_cooldown.value == self.timers.attack_cooldown.initial_value()
        {
            self.current_event = enemy_state_machine::EnemyEvent::InAttackRange;
            self.timers.attack_cooldown.value -= delta;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn fall(&mut self) {
        self.velocity.y = Vector2::DOWN.y * self.agro_speed;
        let velocity = self.velocity;

        // TODO: ??
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        if self.base().is_on_floor() {
            self.state.handle(&enemy_state_machine::EnemyEvent::OnFloor);
        }
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }
}

#[godot_dyn]
impl CharacterResources for TestEnemy {
    fn get_health(&self) -> u32 {
        self.health
    }

    fn set_health(&mut self, amount: u32) {
        self.health = amount;
    }

    fn get_energy(&self) -> u32 {
        self.energy
    }

    fn set_energy(&mut self, amount: u32) {
        self.energy = amount;
    }

    fn get_mana(&self) -> u32 {
        self.mana
    }

    fn set_mana(&mut self, amount: u32) {
        self.mana = amount;
    }
}

impl crate::traits::components::character_components::has_hitbox::HasEnemyHitbox for TestEnemy {}

impl crate::traits::components::character_components::has_state::HasState for TestEnemy {
    fn get_mut_sm(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn get_sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl crate::traits::components::character_components::has_aggro::HasAggroArea for TestEnemy {}

#[godot_dyn]
impl Damageable for TestEnemy {
    fn take_damage(&mut self, amount: u32) {
        let mut current_health = self.get_health();

        current_health = current_health.saturating_sub(amount);
        self.set_health(current_health);

        if self.is_dead() {
            self.destroy();
        }
    }

    fn destroy(&mut self) {
        self.signals().test_enemy_died().emit();
        self.base_mut().queue_free();
    }
}

impl crate::traits::components::character_components::animatable::Animatable for TestEnemy {
    fn get_anim_player(&self) -> Gd<AnimationPlayer> {
        self.animation_player.clone()
    }

    fn get_direction(&self) -> PlatformerDirection {
        self.direction.clone()
    }
}

impl crate::traits::components::character_components::moveable::MoveableCharacter for TestEnemy {}
