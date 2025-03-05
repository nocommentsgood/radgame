use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, Marker2D, RayCast2D, Timer,
    },
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    classes::characters::main_character::MainCharacter,
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
    pub current_event: enemy_state_machine::EnemyEvent,
    delta: f64,
    direction: PlatformerDirection,
    #[init(node = "AttackRayCast")]
    #[var]
    attack_raycast: OnReady<Gd<RayCast2D>>,
    #[init(val = 1.0)]
    attack_animation_timer: f64,
    #[init(val = 3.5)]
    attack_cooldown_timer: f64,
    #[init(val = 2.0)]
    #[var]
    idle_time: f64,
    #[init(val = 40.0)]
    #[var]
    patrol_speed: real,
    #[init(val = 4.0)]
    #[var]
    patrol_time: f64,
    #[init(val = 80.0)]
    #[var]
    agro_speed: real,
    #[var]
    speed: real,
    #[init(node = "LeftPatrolMarker")]
    #[var]
    left_patrol_marker: OnReady<Gd<Marker2D>>,
    #[init(node = "RightPatrolMarker")]
    #[var]
    right_patrol_marker: OnReady<Gd<Marker2D>>,
    #[init(val = 30)]
    #[var]
    health: i32,
    #[var]
    energy: i32,
    #[var]
    mana: i32,
    #[var]
    velocity: Vector2,
    state: statig::blocking::StateMachine<EnemyStateMachine>,

    #[init(node = "AnimationPlayer2")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[init(node = "MovementTimer")]
    movement_timer: OnReady<Gd<Timer>>,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {
        self.connect_player_sensors();

        let callable = self.base().callable(constants::CALLABLE_DESTROY_ENEMY);
        self.base_mut()
            .connect(constants::SIGNAL_TESTENEMY_DIED, &callable);
    }

    fn physics_process(&mut self, delta: f64) {
        self.delta = delta;
        self.state.handle(&self.current_event);

        match self.state.state() {
            enemy_state_machine::State::Idle {} => self.idle(),
            enemy_state_machine::State::ChasePlayer { player } => self.chase_player(player.clone()),
            enemy_state_machine::State::Patrol {} => self.patrol(),
            enemy_state_machine::State::Attack { player } => self.attack(player.clone()),
        }
        self.update_animation();
        self.raycast_stuff();
    }
}

#[godot_api]
impl TestEnemy {
    #[signal]
    fn test_enemy_died();

    #[func]
    fn destroy(&mut self) {
        if self.is_dead() {
            self.base_mut().queue_free();
        }
    }

    #[func]
    fn on_enemy_senses_player(&mut self, body: Gd<Node2D>) {
        if body.is_in_group("player") {
            if let Ok(player) = body.try_cast::<MainCharacter>() {
                self.current_event = enemy_state_machine::EnemyEvent::FoundPlayer {
                    player: player.clone(),
                }
            }
        }
    }

    fn raycast_stuff(&mut self) {
        let raycast = self.get_attack_raycast();
        if let Some(collider) = raycast.get_collider() {
            println!("Got collision: {}", collider);
            // attack logic
        }
    }

    fn connect_player_sensors(&mut self) {
        // Connect to aggro range
        let mut player_sensors = self.base().get_node_as::<Area2D>(constants::PLAYER_SENSORS);
        let callable = self
            .base()
            .callable(constants::CALLABLE_ENEMY_SENSES_PLAYER);
        player_sensors.connect(constants::SIGNAL_ENEMY_DETECTS_PLAYER, &callable);

        // Connect to player entering attack range
        let call = self
            .base()
            .callable(constants::CALLABLE_PLAYER_ENTERED_ATTACK_RANGE);
        // player_sensors.connect(constants::SIGNAL_PLAYER_ENTERED_ATTACK_RANGE, &call);

        player_sensors.connect("area_shape_entered", &call);
    }

    fn get_furthest_patrol_marker(&self) -> Vector2 {
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

    fn reset_attack_animation_timer(&mut self) {
        self.attack_animation_timer = 1.0;
    }

    fn reset_attack_cooldown_timer(&mut self) {
        self.attack_cooldown_timer = 3.5;
    }

    fn attack(&mut self, player: Gd<MainCharacter>) {
        self.base_mut().set_velocity(Vector2::ZERO);
        self.velocity = Vector2::ZERO;
        let mut time = self.attack_animation_timer;
        time -= self.delta;

        if time <= 0.0 {
            self.reset_attack_animation_timer();
        }
    }

    pub fn patrol(&mut self) {
        let time = self.get_patrol_time();
        let speed = self.patrol_speed;
        let velocity = self.get_velocity();

        self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();
        self.set_patrol_time(time - self.delta);

        if time <= 0.0 {
            self.reset_patrol_time();
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    pub fn idle(&mut self) {
        let time = self.get_idle_time();
        self.direction = PlatformerDirection::from_platformer_velocity(&Vector2::ZERO);
        self.velocity = Vector2::ZERO;
        self.base_mut().set_velocity(Vector2::ZERO);
        self.set_idle_time(time - self.delta);

        if time <= 0.0 {
            self.reset_idle_time();

            self.set_velocity(self.get_furthest_patrol_marker());
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    pub fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let speed = self.agro_speed;
        let player_position = player.get_position();
        let velocity = self
            .base()
            .get_position()
            .direction_to(player_position)
            .normalized_or_zero();

        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

        self.current_event = enemy_state_machine::EnemyEvent::None;
    }

    fn reset_idle_time(&mut self) {
        self.set_idle_time(2.0);
    }

    fn reset_patrol_time(&mut self) {
        self.set_patrol_time(4.0);
    }

    fn get_current_animation(&self) -> String {
        let direciton = &self.direction;
        let mut state = self.state.state().to_string();
        state.push('_');

        format!("{}{}", state, direciton)
    }

    fn update_animation(&mut self) {
        let animation = self.get_current_animation();
        self.animation_player.play_ex().name(&animation).done();
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
            self.base_mut()
                .emit_signal(constants::SIGNAL_TESTENEMY_DIED, &[]);
        }
    }
}

#[godot_dyn]
impl Damaging for TestEnemy {}
