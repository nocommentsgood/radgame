use std::collections::HashMap;

use bevy_time::{Timer, TimerMode};
use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D},
    obj::WithBaseField,
    prelude::*,
};

use super::{
    animatable::Animatable,
    enemy_state_ext::EnemyCharacterStateMachineExt,
    enemy_state_machine::{EnemyEvent, EnemyStateMachine, State},
    has_enemy_sensors::HasEnemySensors,
    has_state::HasState,
    patrol_component::PatrolComp,
};
use crate::entities::{
    damage::Damageable,
    entity_stats::EntityResources,
    movements::MoveableCharacter,
    movements::{Direction, SpeedComponent},
    time::EnemyTimer,
};

type ET = EnemyTimer;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    direction: Direction,
    velocity: Vector2,
    timers: HashMap<EnemyTimer, Timer>,
    speeds: SpeedComponent,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    base: Base<CharacterBody2D>,
    energy: u32,
    mana: u32,
    player_pos: Option<Vector2>,
    #[init(val = 100)]
    health: u32,
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    patrol_comp: PatrolComp,
    #[export]
    #[export_subgroup(name = "PatrolComponent")]
    left_target: Vector2,
    #[export]
    #[export_subgroup(name = "PatrolComponent")]
    right_target: Vector2,

    #[init(node = "NavigationAgent2D")]
    nav_agent: OnReady<Gd<godot::classes::NavigationAgent2D>>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {
        self.patrol_comp.left_target = self.left_target;
        self.patrol_comp.right_target = self.right_target;
        self.speeds = SpeedComponent::new(40, 40, 80);
        self.connect_signals();

        self.timers.insert(
            ET::AttackAnimation,
            Timer::from_seconds(1.8, TimerMode::Once),
        );
        self.timers.insert(
            ET::AttackCooldown,
            Timer::from_seconds(2.0, TimerMode::Once),
        );
        self.timers.insert(
            ET::AttackChainCooldown,
            Timer::from_seconds(2.7, TimerMode::Once),
        );
        self.timers
            .insert(ET::Idle, Timer::from_seconds(2.7, TimerMode::Once));
        self.timers
            .insert(ET::Patrol, Timer::from_seconds(3.0, TimerMode::Once));

        self.hurtbox_mut().bind_mut().attack_damage = 10;
    }

    fn physics_process(&mut self, _delta: f64) {
        self.check_floor();
        // dbg!(&self.state.state());

        match self.state.state() {
            State::Idle {} => self.idle(),
            State::ChasePlayer {} => self.chase_player(),
            State::Patrol {} => self.patrol(),
            State::Attack {} => self.attack(),
            State::Attack2 {} => self.chain_attack(),
            State::Falling {} => self.fall(),
        }

        // self.update_timers();
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
            self.state.handle(&EnemyEvent::FailedFloorCheck);
        }
    }

    // Leaving this somewhat open ended in case more timers are added later
    // fn update_timers(&mut self) {
    //     let delta = self.base().get_physics_process_delta_time() as f32;
    //     let ac = &ET::AttackChainCooldown;
    //
    //     // Update attack cooldown timer
    //     let attack_cooldown = self.timers.get(ac);
    //     if attack_cooldown < self.timers.get_init(ac) && attack_cooldown > 0.0 {
    //         self.timers.set(ac, attack_cooldown - delta);
    //     } else if attack_cooldown <= 0.0 {
    //         self.timers.reset(ac);
    //     }
    // }
}

#[godot_dyn]
impl EntityResources for TestEnemy {
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

impl HasState for TestEnemy {
    fn sm_mut(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl HasEnemySensors for TestEnemy {
    fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>) {
        self.player_pos = pos;
    }
}

#[godot_dyn]
impl Damageable for TestEnemy {
    fn destroy(&mut self) {
        self.signals().test_enemy_died().emit();
        self.base_mut().queue_free();
    }
}

impl Animatable for TestEnemy {
    fn get_anim_player(&mut self) -> &mut Gd<AnimationPlayer> {
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

impl MoveableCharacter for TestEnemy {}

impl EnemyCharacterStateMachineExt for TestEnemy {
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

    fn get_player_pos(&self) -> Option<Vector2> {
        self.player_pos
    }
}
