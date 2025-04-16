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
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
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
    health: i32,

    #[var]
    energy: i32,

    #[var]
    mana: i32,

    #[var]
    velocity: Vector2,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {
        self.connect_signals();
        self.timers = EnemyTimers::new(1.8, 2.0, 2.7, 2.0, 4.0);
    }

    fn physics_process(&mut self, _delta: f64) {
        self.state.handle(&self.current_event);
        self.check_floor();

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

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        damaging.dyn_bind().do_damage(damageable);
    }

    fn on_aggro_area_entered(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            if let Some(player) = area.get_parent() {
                if let Ok(player) = player.try_cast::<MainCharacter>() {
                    self.current_event = enemy_state_machine::EnemyEvent::FoundPlayer {
                        player: player.clone(),
                    }
                }
            }
        }
    }

    fn on_aggro_area_exited(&mut self, area: Gd<Area2D>) {
        if area.is_in_group("player") {
            self.current_event = enemy_state_machine::EnemyEvent::LostPlayer;
        }
    }

    fn connect_signals(&mut self) {
        let player_sensors = self.base().get_node_as::<Node2D>(constants::ENEMY_SENSORS);
        let mut aggro_area = player_sensors.get_node_as::<Area2D>("AggroArea");
        let mut hitbox = player_sensors.get_node_as::<Area2D>("Hitbox");

        // connect hitbox to entering areas
        let mut this = self.to_gd();
        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));

        // connect to player enters aggro range
        let mut this = self.to_gd();
        aggro_area
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_aggro_area_entered(area));

        // Connect to player leaves aggro range
        let mut this = self.to_gd();
        aggro_area
            .signals()
            .area_exited()
            .connect(move |area| this.bind_mut().on_aggro_area_exited(area));
    }

    fn furthest_patrol_marker_distance(&self) -> Vector2 {
        let left = self.get_left_patrol_marker();
        let right = self.get_right_patrol_marker();
        let left_distance = self
            .base()
            .get_global_position()
            .distance_to(left.get_global_position());
        let right_distance = self
            .base()
            .get_global_position()
            .distance_to(right.get_global_position());

        let target = if left_distance <= right_distance {
            left
        } else {
            right
        };

        let velocity = self
            .base()
            .get_global_position()
            .direction_to(target.get_global_position())
            .normalized_or_zero();
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
        self.update_animation();
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

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
        self.update_animation();
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

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

        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();
        self.timers.patrol.value -= delta;

        if time <= 0.0 {
            self.timers.patrol.reset();
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn idle(&mut self) {
        let time = self.timers.idle.value;
        let delta = self.base().get_physics_process_delta_time();
        // self.velocity = Vector2::ZERO;
        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(Vector2::ZERO);
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
        self.update_animation();
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

        if attack_range.has_overlapping_bodies()
            && self.timers.attack_cooldown.value == self.timers.attack_cooldown.initial_value()
        {
            self.current_event = enemy_state_machine::EnemyEvent::InAttackRange;
            self.timers.attack_cooldown.value -= delta;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn get_current_animation(&self) -> String {
        let direciton = &self.direction;
        let mut state = self.state.state().to_string();
        state.push('_');

        format!("{}{}", state, direciton)
    }

    fn fall(&mut self) {
        self.velocity.y = Vector2::DOWN.y * self.agro_speed;
        let velocity = self.velocity;
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();

        if self.base().is_on_floor() {
            self.state.handle(&enemy_state_machine::EnemyEvent::OnFloor);
        }
    }

    fn update_animation(&mut self) {
        let animation = self.get_current_animation();
        self.animation_player.play_ex().name(&animation).done();
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }
}

#[godot_dyn]
impl CharacterResources for TestEnemy {
    fn get_health(&self) -> i32 {
        self.health
    }

    fn set_health(&mut self, amount: i32) {
        self.health = amount;
    }

    fn get_energy(&self) -> i32 {
        self.energy
    }

    fn set_energy(&mut self, amount: i32) {
        self.energy = amount;
    }

    fn get_mana(&self) -> i32 {
        self.mana
    }

    fn set_mana(&mut self, amount: i32) {
        self.mana = amount;
    }
}

#[godot_dyn]
impl Damageable for TestEnemy {
    fn take_damage(&mut self, amount: i32) {
        let mut current_health = self.get_health();

        current_health = current_health.saturating_sub(amount);
        self.set_health(current_health);

        if self.is_dead() {
            self.signals().test_enemy_died().emit();
            self.base_mut().queue_free();
        }
    }
}
