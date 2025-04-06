use godot::{
    classes::{AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, Marker2D},
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
    current_event: enemy_state_machine::EnemyEvent,
    delta: f64,
    direction: PlatformerDirection,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    base: Base<CharacterBody2D>,

    #[init(val = 1.0)]
    attack_animation_timer: f64,

    #[init(val = 3.5)]
    attack_cooldown_timer: f64,

    #[var]
    #[init(val = 2.0)]
    idle_time: f64,

    #[var]
    #[init(val = 40.0)]
    patrol_speed: real,

    #[var]
    #[init(val = 4.0)]
    patrol_time: f64,

    #[var]
    #[init(val = 80.0)]
    agro_speed: real,

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
    #[init(val = 30)]
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
        self.connect_area_nodes();
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
        self.update_timers();
    }
}

#[godot_api]
impl TestEnemy {
    #[signal]
    fn test_enemy_died();

    #[signal]
    fn can_attack_player();

    fn destroy(&mut self) {
        if self.is_dead() {
            self.signals().test_enemy_died().emit();
            self.base_mut().queue_free();
        }
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        println!("area entered enemy hitbox");
        let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
        let target = self.to_gd().upcast::<Node2D>();
        let _guard = self.base_mut();
        let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
        damaging.dyn_bind().do_damage(damageable);
    }

    fn on_aggro_area_entered(&mut self, body: Gd<Node2D>) {
        if body.is_in_group("player") {
            if let Ok(player) = body.try_cast::<MainCharacter>() {
                self.current_event = enemy_state_machine::EnemyEvent::FoundPlayer {
                    player: player.clone(),
                }
            }
        }
    }

    fn on_aggro_area_exited(&mut self, body: Gd<Node2D>) {
        if body.is_in_group("player") {
            self.current_event = enemy_state_machine::EnemyEvent::LostPlayer;
        }
    }

    fn connect_area_nodes(&mut self) {
        let sensors = self.base().get_node_as::<Node2D>(constants::ENEMY_SENSORS);
        let mut aggro_area = sensors.get_node_as::<Area2D>("AggroArea");
        let mut hitbox = sensors.get_node_as::<Area2D>("Hitbox");
        let mut this = self.to_gd();
        println!("hitbox is: {}", hitbox);

        hitbox
            .signals()
            .area_entered()
            .connect(move |area| this.bind_mut().on_area_entered_hitbox(area));

        // connect to player enters aggro range
        let mut this = self.to_gd();
        aggro_area
            .signals()
            .body_entered()
            .connect(move |body| this.bind_mut().on_aggro_area_entered(body));

        // Connect to player leaves aggro range
        let mut this = self.to_gd();
        aggro_area
            .signals()
            .body_exited()
            .connect(move |body| this.bind_mut().on_aggro_area_exited(body));
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

    fn reset_attack_animation_timer(&mut self) {
        self.attack_animation_timer = 1.0;
    }

    fn reset_attack_cooldown_timer(&mut self) {
        self.attack_cooldown_timer = 3.5;
    }

    // Leaving this somewhat open ended in case more timers are added later
    fn update_timers(&mut self) {
        // Update attack cooldown timer
        if self.attack_cooldown_timer < 3.5 && self.attack_cooldown_timer > 0.0 {
            self.attack_cooldown_timer -= self.delta;
        } else if self.attack_cooldown_timer <= 0.0 {
            self.reset_attack_cooldown_timer();
        }
    }

    fn attack(&mut self, _player: Gd<MainCharacter>) {
        self.base_mut().set_velocity(Vector2::ZERO);
        self.velocity = Vector2::ZERO;
        self.attack_animation_timer -= self.delta;

        if self.attack_animation_timer <= 0.0 {
            self.reset_attack_animation_timer();
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        }
    }

    fn patrol(&mut self) {
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

    fn idle(&mut self) {
        let time = self.get_idle_time();
        self.direction = PlatformerDirection::from_platformer_velocity(&Vector2::ZERO);
        self.velocity = Vector2::ZERO;
        self.base_mut().set_velocity(Vector2::ZERO);
        self.set_idle_time(time - self.delta);

        if time <= 0.0 {
            self.reset_idle_time();

            self.set_velocity(self.furthest_patrol_marker_distance());
            self.current_event = enemy_state_machine::EnemyEvent::TimerElapsed;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
    }

    fn chase_player(&mut self, player: Gd<MainCharacter>) {
        let attack_range = self.base().get_node_as::<Area2D>("EnemySensors/AttackArea");
        let speed = self.agro_speed;
        let player_position = player.get_position();
        let velocity = self
            .base()
            .get_position()
            .direction_to(player_position)
            .normalized_or_zero();

        self.direction = PlatformerDirection::from_platformer_velocity(&velocity);
        self.base_mut().set_velocity(velocity * speed);
        self.base_mut().move_and_slide();

        if attack_range.has_overlapping_bodies() && self.attack_cooldown_timer == 3.5 {
            self.current_event = enemy_state_machine::EnemyEvent::InAttackRange;
            self.attack_cooldown_timer -= self.delta;
        } else {
            self.current_event = enemy_state_machine::EnemyEvent::None;
        }
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
            self.destroy();
        }
    }
}
