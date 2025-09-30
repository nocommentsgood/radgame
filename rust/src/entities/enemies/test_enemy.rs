use std::{collections::HashMap, ops::Deref};

use godot::{
    classes::{AnimationPlayer, Area2D, CharacterBody2D, ICharacterBody2D, RayCast2D, Timer},
    obj::WithBaseField,
    prelude::*,
};

use super::{
    animatable::Animatable,
    enemy_state_ext::EnemyEntityStateMachineExt,
    enemy_state_machine::{EnemyEvent, EnemyStateMachine, State},
    has_enemy_sensors::HasEnemySensors,
    has_state::HasState,
};
use crate::entities::{
    damage::{AttackData, Damage, DamageType, Damageable, HasHealth},
    enemies::{
        has_enemy_sensors::{EnemySensors, PlayerDetection},
        physics::{self, FrameData, PatrolComp},
        time::{EnemyTimers, Timers},
    },
    ent_graphics::EntGraphics,
    hit_reg::{HitReg, Hitbox, Hurtbox},
    movements::{Direction, Move, Moveable, MoveableBody},
    time::EnemyTimer,
};

type ET = EnemyTimer;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    // health: u32,
    // previous_velocity: Vector2,
    // chain_attack_count: u32,
    // timers: HashMap<EnemyTimer, Gd<Timer>>,
    // state: statig::blocking::StateMachine<EnemyStateMachine>,
    // base: Base<CharacterBody2D>,
    // player_pos: Option<Vector2>,
    // #[init(node = "AnimationPlayer")]
    // animation_player: OnReady<Gd<AnimationPlayer>>,
    //
    // patrol_comp: PatrolComp,
    // #[export]
    // #[export_subgroup(name = "PatrolComponent")]
    // left_target: Vector2,
    // #[export]
    // #[export_subgroup(name = "PatrolComponent")]
    // right_target: Vector2,
    //
    // #[init(node = "NavigationAgent2D")]
    // nav_agent: OnReady<Gd<godot::classes::NavigationAgent2D>>,
    //
    // velocity: Vector2,
    // direction: Direction,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {}
}
//         self.hurtbox_mut().bind_mut().data = Some(AttackData {
//             hurtbox: self.hurtbox().clone(),
//             parryable: true,
//             damage: Damage {
//                 raw: 10,
//                 d_type: DamageType::Physical,
//             },
//         });
//
//         self.hitbox_mut().bind_mut().damageable_parent = Some(Box::new(self.to_gd()));
//
//         self.patrol_comp.left_target = self.left_target;
//         self.patrol_comp.right_target = self.right_target;
//
//         self.timers
//             .insert(ET::AttackAnimation, godot::classes::Timer::new_alloc());
//         self.timers
//             .insert(ET::AttackCooldown, godot::classes::Timer::new_alloc());
//         self.timers
//             .insert(ET::AttackChainCooldown, godot::classes::Timer::new_alloc());
//         self.timers
//             .insert(ET::Idle, godot::classes::Timer::new_alloc());
//         self.timers
//             .insert(ET::Patrol, godot::classes::Timer::new_alloc());
//
//         self.timers
//             .get_mut(&ET::AttackAnimation)
//             .unwrap()
//             .set_wait_time(1.7);
//         self.timers
//             .get_mut(&ET::AttackCooldown)
//             .unwrap()
//             .set_wait_time(2.0);
//         self.timers
//             .get_mut(&ET::AttackChainCooldown)
//             .unwrap()
//             .set_wait_time(1.35);
//         self.timers.get_mut(&ET::Idle).unwrap().set_wait_time(1.5);
//
//         let mut ts = self.timers.clone();
//         ts.values_mut().for_each(|timer| {
//             timer.set_one_shot(true);
//             self.base_mut().add_child(&timer.clone());
//         });
//
//         self.connect_signals();
//         self.idle();
//         self.animation_player.play_ex().name("idle_east").done();
//     }
//
//     fn physics_process(&mut self, _delta: f64) {
//         self.raycast_check();
//
//         if self.previous_velocity.x != self.get_velocity().x {
//             self.previous_velocity = self.get_velocity();
//             self.update_animation();
//         }
//         self.check_floor();
//         match self.state.state() {
//             State::Falling {} => self.fall(),
//             // TODO: Add navigationagent.
//             State::ChasePlayer {} => self.chase_player(),
//             State::Patrol {} => self.patrol(),
//             State::Attack2 {} => self.track_player(),
//             _ => (),
//         }
//     }
// }

// #[godot_api]
// impl TestEnemy {
//     #[signal]
//     pub fn test_enemy_died();
//
//     fn check_floor(&mut self) {
//         if !self.base().is_on_floor() {
//             self.transition_sm(&EnemyEvent::FailedFloorCheck);
//         }
//     }
// }
//
// impl HasState for TestEnemy {
//     fn sm_mut(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
//         &mut self.state
//     }
//
//     fn sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
//         &self.state
//     }
// }
//
// impl HasEnemySensors for TestEnemy {
//     fn set_player_pos(&mut self, pos: Option<godot::builtin::Vector2>) {
//         self.player_pos = pos;
//     }
// }
//
// impl Animatable for TestEnemy {
//     fn anim_player_mut(&mut self) -> &mut Gd<AnimationPlayer> {
//         &mut self.animation_player
//     }
//
//     fn get_direction(&self) -> &Direction {
//         &self.direction
//     }
//
//     fn update_direction(&mut self) {
//         if !self.velocity.x.is_zero_approx() {
//             self.direction = Direction::from_vel(&self.velocity);
//         }
//     }
// }
//
// impl Moveable for TestEnemy {
//     fn get_velocity(&self) -> Vector2 {
//         self.velocity
//     }
//
//     fn set_velocity(&mut self, velocity: Vector2) {
//         self.velocity = velocity;
//     }
// }
//
// impl MoveableBody for TestEnemy {
//     fn notify_on_floor(&mut self) {
//         self.transition_sm(&EnemyEvent::OnFloor);
//     }
// }
//
// impl Move for TestEnemy {
//     fn slide(&mut self) {
//         self.phy_slide()
//     }
// }
//
// impl HasHealth for Gd<TestEnemy> {
//     fn get_health(&self) -> u32 {
//         self.bind().health
//     }
//
//     fn set_health(&mut self, amount: u32) {
//         self.bind_mut().health = amount;
//     }
//
//     fn on_death(&mut self) {
//         self.signals().test_enemy_died().emit();
//         self.queue_free();
//     }
// }
//
// impl Damageable for Gd<TestEnemy> {
//     fn handle_attack(&mut self, attack: AttackData) {
//         self.take_damage(attack.damage.raw);
//     }
// }
//
// impl EnemyEntityStateMachineExt for TestEnemy {
//     fn timers(&mut self) -> &mut HashMap<EnemyTimer, Gd<Timer>> {
//         &mut self.timers
//     }
//
//     fn speeds(&self) -> &EnemySpeeds {
//         &self.speeds
//     }
//
//     fn patrol_comp(&self) -> &PatrolComp {
//         &self.patrol_comp
//     }
//
//     fn get_player_pos(&self) -> Option<Vector2> {
//         self.player_pos
//     }
//
//     fn get_chain_attack_count(&self) -> u32 {
//         self.chain_attack_count
//     }
//
//     fn set_chain_attack_count(&mut self, amount: u32) {
//         self.chain_attack_count = amount;
//     }
//
//     // TODO: Implement
//     fn attack_implementation(&mut self) {
//         unimplemented!("Attack types")
//     }
// }

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct ModularTestEnemy {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::manual())]
    movement: OnReady<physics::Movement>,

    #[init(val = OnReady::from_base_fn(EntGraphics::new))]
    graphics: OnReady<EntGraphics>,

    #[init(val = OnReady::from_base_fn(EnemySensors::new))]
    sensors: OnReady<EnemySensors>,

    #[init(val = OnReady::from_base_fn(Timers::new))]
    timers: OnReady<Timers>,

    patrol: PatrolComp,
    sm: statig::blocking::StateMachine<EnemyStateMachine>,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for ModularTestEnemy {
    fn ready(&mut self) {
        self.patrol.right_target = self.right_target;
        self.patrol.left_target = self.left_target;
        self.movement.init(physics::Movement::new(
            physics::Speeds::new(100.0, 150.0),
            Vector2::default(),
        ));
        self.sensors.hit_reg.hurtbox.bind_mut().data = Some(AttackData {
            hurtbox: self.sensors.hit_reg.hurtbox.clone(),
            parryable: true,
            damage: Damage {
                raw: 10,
                d_type: DamageType::Physical,
            },
        });

        self.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_aggro_area_entered);
        self.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_exited()
            .connect_other(&self.to_gd(), Self::on_aggro_area_exited);
        self.sensors
            .player_detection
            .attack_area
            .signals()
            .area_entered()
            .connect_other(&self.to_gd(), Self::on_attack_area_entered);

        self.timers
            .idle
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_idle_timeout);
        self.timers
            .patrol
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_patrol_timeout);
        self.timers
            .attack
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_attack_timeout);

        self.timers.idle.start();
    }

    fn physics_process(&mut self, delta: f32) {
        if self.sensors.is_any_raycast_colliding() {
            self.sm.handle(&EnemyEvent::RayCastNotColliding);
        }

        if !self.base().is_on_floor() {
            self.sm.handle(&EnemyEvent::FailedFloorCheck);
        }

        match self.sm.state() {
            State::ChasePlayer {} => {
                self.sensors.player_detection.track_player_position();
                if self
                    .sensors
                    .player_detection
                    .attack_area
                    .has_overlapping_areas()
                    && self.timers.attack.get_time_left() == 0.0
                {
                    self.timers.attack.start();
                    self.sm.handle(&EnemyEvent::InAttackRange);
                }
            }
            State::Falling {} => {
                if self.base().is_on_floor() {
                    self.sm.handle(&EnemyEvent::OnFloor);
                }
            }
            _ => (),
        }

        let frame = FrameData::new(
            self.sm.state(),
            self.base().is_on_floor(),
            self.base().get_global_position(),
            self.sensors.player_position(),
            &self.patrol,
            delta,
        );

        self.graphics.update(
            self.sm.state(),
            &Direction::from_vel(&self.movement.velocity),
        );
        self.movement.update(&frame);
        let v = self.movement.velocity;
        self.base_mut().set_velocity(v);
        self.base_mut().move_and_slide();

        dbg!(&self.sm.state());
    }
}

impl ModularTestEnemy {
    fn on_aggro_area_entered(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&EnemyEvent::FoundPlayer);
    }

    fn on_aggro_area_exited(&mut self, _area: Gd<Area2D>) {
        self.sm.handle(&EnemyEvent::LostPlayer);
    }

    fn on_attack_area_entered(&mut self, _area: Gd<Area2D>) {
        if self.timers.attack.get_time_left() == 0.0 {
            self.timers.attack.start();
            self.sm.handle(&EnemyEvent::InAttackRange);
        }
    }

    fn on_idle_timeout(&mut self) {
        let frame = FrameData::new(
            self.sm.state(),
            self.base().is_on_floor(),
            self.base().get_global_position(),
            self.sensors.player_position(),
            &self.patrol,
            self.base().get_physics_process_delta_time() as f32,
        );
        if self.sm.state() == (&State::Idle {}) {
            self.timers.patrol.start();
            self.movement.update_patrol_target(&frame);

            self.sm.handle(&EnemyEvent::TimerElapsed(EnemyTimers::Idle));
        }
    }

    fn on_patrol_timeout(&mut self) {
        if self.sm.state() == (&State::Patrol {}) {
            self.movement.stop();
            self.timers.idle.start();
            self.sm
                .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Patrol));
        }
    }

    fn on_attack_timeout(&mut self) {
        self.sm
            .handle(&EnemyEvent::TimerElapsed(EnemyTimers::Attack));
    }
}
