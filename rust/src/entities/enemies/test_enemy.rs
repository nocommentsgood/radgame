use std::collections::HashMap;

use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D, Timer},
    obj::WithBaseField,
    prelude::*,
};

use super::{
    animatable::Animatable,
    enemy_state_ext::EnemyEntityStateMachineExt,
    enemy_state_machine::{EnemyEvent, EnemyStateMachine, State},
    has_enemy_sensors::HasEnemySensors,
    has_state::HasState,
    patrol_component::PatrolComp,
};
use crate::entities::{
    damage::{AttackData, Damage, DamageType, Damageable, HasHealth},
    movements::{Direction, Move, Moveable, MoveableBody, SpeedComponent},
    time::EnemyTimer,
};

type ET = EnemyTimer;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    health: u32,
    previous_velocity: Vector2,
    chain_attack_count: u32,
    direction: Direction,
    velocity: Vector2,
    timers: HashMap<EnemyTimer, Gd<Timer>>,
    speeds: SpeedComponent,
    state: statig::blocking::StateMachine<EnemyStateMachine>,
    base: Base<CharacterBody2D>,
    player_pos: Option<Vector2>,
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
        self.hurtbox_mut().bind_mut().data = Some(AttackData {
            hurtbox: self.hurtbox().clone(),
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });

        self.hitbox_mut().bind_mut().damageable_parent = Some(Box::new(self.to_gd()));

        self.patrol_comp.left_target = self.left_target;
        self.patrol_comp.right_target = self.right_target;
        self.speeds = SpeedComponent::new(40, 40, 80);

        self.timers
            .insert(ET::AttackAnimation, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::AttackCooldown, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::AttackChainCooldown, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::Idle, godot::classes::Timer::new_alloc());
        self.timers
            .insert(ET::Patrol, godot::classes::Timer::new_alloc());

        self.timers
            .get_mut(&ET::AttackAnimation)
            .unwrap()
            .set_wait_time(1.7);
        self.timers
            .get_mut(&ET::AttackCooldown)
            .unwrap()
            .set_wait_time(2.0);
        self.timers
            .get_mut(&ET::AttackChainCooldown)
            .unwrap()
            .set_wait_time(1.35);
        self.timers.get_mut(&ET::Idle).unwrap().set_wait_time(1.5);

        let mut ts = self.timers.clone();
        ts.values_mut().for_each(|timer| {
            timer.set_one_shot(true);
            self.base_mut().add_child(&timer.clone());
        });

        self.connect_signals();
        self.idle();
        self.animation_player.play_ex().name("idle_east").done();
    }

    fn physics_process(&mut self, _delta: f64) {
        self.raycast_check();

        if self.previous_velocity.x != self.get_velocity().x {
            self.previous_velocity = self.get_velocity();
            self.update_animation();
        }
        self.check_floor();
        match self.state.state() {
            State::Falling {} => self.fall(),
            // TODO: Add navigationagent.
            State::ChasePlayer {} => self.chase_player(),
            State::Patrol {} => self.patrol(),
            State::Attack2 {} => self.track_player(),
            _ => (),
        }
        // dbg!(&self.state.state());

        // self.update_timers();
    }
}

#[godot_api]
impl TestEnemy {
    #[signal]
    pub fn test_enemy_died();

    fn check_floor(&mut self) {
        if !self.base().is_on_floor() {
            self.transition_sm(&EnemyEvent::FailedFloorCheck);
        }
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

impl Animatable for TestEnemy {
    fn anim_player_mut(&mut self) -> &mut Gd<AnimationPlayer> {
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

impl Moveable for TestEnemy {
    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}

impl MoveableBody for TestEnemy {
    fn notify_on_floor(&mut self) {
        self.transition_sm(&EnemyEvent::OnFloor);
    }
}

impl Move for TestEnemy {
    fn slide(&mut self) {
        self.phy_slide()
    }
}

impl HasHealth for Gd<TestEnemy> {
    fn get_health(&self) -> u32 {
        self.bind().health
    }

    fn set_health(&mut self, amount: u32) {
        self.bind_mut().health = amount;
    }

    fn on_death(&mut self) {
        self.signals().test_enemy_died().emit();
        self.queue_free();
    }
}

impl Damageable for Gd<TestEnemy> {
    fn handle_attack(&mut self, attack: AttackData) {
        self.take_damage(attack.damage.raw);
    }
}

impl EnemyEntityStateMachineExt for TestEnemy {
    fn timers(&mut self) -> &mut HashMap<EnemyTimer, Gd<Timer>> {
        &mut self.timers
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

    fn get_chain_attack_count(&self) -> u32 {
        self.chain_attack_count
    }

    fn set_chain_attack_count(&mut self, amount: u32) {
        self.chain_attack_count = amount;
    }

    // TODO: Implement
    fn attack_implementation(&mut self) {
        unimplemented!("Attack types")
        // if let Some(p) = self.get_player_pos() {
        //     let pos = self.base().get_position();
        //     let dir = Direction::from_vel(&pos.direction_to(p));
        //     match dir {
        //         Direction::East => self.animation_player.play_ex().name("attack_east").done(),
        //         Direction::West => self.animation_player.play_ex().name("attack_west").done(),
        //     }
        // }
    }
}
