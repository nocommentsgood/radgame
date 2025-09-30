use godot::{
    classes::{Area2D, CharacterBody2D, ICharacterBody2D},
    obj::WithBaseField,
    prelude::*,
};

use super::enemy_state_machine::{EnemyEvent, EnemyStateMachine, State};
use crate::entities::{
    damage::{AttackData, Damage, DamageType},
    enemies::{
        enemy_state_machine::EnemySMType,
        has_enemy_sensors::EnemySensors,
        physics::{self, Movement, PatrolComp, PhysicsFrameData, Speeds},
        time::{EnemyTimers, Timers},
    },
    ent_graphics::EntGraphics,
    movements::Direction,
};

#[derive(Clone)]
struct EnemyEntity {
    movement: Movement,
    graphics: EntGraphics,
    sensors: EnemySensors,
    timers: Timers,
    patrol: PatrolComp,
    sm: EnemySMType,
}

impl EnemyEntity {
    pub fn new(
        node: &Gd<Node>,
        speeds: Speeds,
        left_patrol_target: Vector2,
        right_patrol_target: Vector2,
        sm: EnemySMType,
    ) -> Self {
        Self {
            movement: Movement::new(speeds, Vector2::default()),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::new(node),
            timers: Timers::new(node),
            patrol: PatrolComp::new(left_patrol_target, right_patrol_target),
            sm,
        }
    }

    pub fn new_and_init(
        node: &Gd<Node>,
        speeds: Speeds,
        left_patrol_target: Vector2,
        right_patrol_target: Vector2,
        sm: EnemySMType,
    ) -> Self {
        let mut this = Self {
            movement: Movement::new(speeds, Vector2::default()),
            graphics: EntGraphics::new(node),
            sensors: EnemySensors::new(node),
            timers: Timers::new(node),
            patrol: PatrolComp::new(left_patrol_target, right_patrol_target),
            sm,
        };
        let mut s = this.clone();
        this.timers
            .attack
            .signals()
            .timeout()
            .connect(move || s.on_attack_timeout());
        let mut s = this.clone();
        this.timers
            .idle
            .signals()
            .timeout()
            .connect(move || s.on_idle_timeout());
        let mut s = this.clone();
        this.timers
            .patrol
            .signals()
            .timeout()
            .connect(move || s.on_patrol_timeout());
        let mut s = this.clone();
        this.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_entered()
            .connect(move |area| s.on_aggro_area_entered(area));
        let mut s = this.clone();
        this.sensors
            .player_detection
            .aggro_area
            .signals()
            .area_exited()
            .connect(move |area| s.on_aggro_area_exited(area));
        let mut s = this.clone();
        this.sensors
            .player_detection
            .attack_area
            .signals()
            .area_entered()
            .connect(move |area| s.on_attack_area_entered(area));
        dbg!(this.timers.idle.start());
        this
    }

    fn physics_process(&mut self, node: &mut Gd<CharacterBody2D>, delta: f32) {
        if self.sensors.is_any_raycast_colliding() {
            self.sm.handle(&EnemyEvent::RayCastNotColliding);
        }

        if !node.is_on_floor() {
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
                if node.is_on_floor() {
                    self.sm.handle(&EnemyEvent::OnFloor);
                }
            }
            _ => (),
        }

        let frame = PhysicsFrameData::new(
            self.sm.state(),
            node.is_on_floor(),
            node.get_global_position(),
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
        node.set_velocity(v);
        node.move_and_slide();

        dbg!(&self.sm.state());
    }

    fn process(&mut self, node: &mut Gd<Node2D>, delta: f32) {}
}

impl EnemyEntity {
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
        // let frame = FrameData::new(
        //     self.sm.state(),
        //     self.base().is_on_floor(),
        //     self.base().get_global_position(),
        //     self.sensors.player_position(),
        //     &self.patrol,
        //     self.base().get_physics_process_delta_time() as f32,
        // );
        if self.sm.state() == (&State::Idle {}) {
            self.timers.patrol.start();
            // self.movement.update_patrol_target(&frame);

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

#[derive(GodotClass)]
#[class(base = CharacterBody2D, init)]
struct ActualEnemyBody {
    #[export]
    left_target: Vector2,
    #[export]
    right_target: Vector2,

    #[init(val = OnReady::from_base_fn(|base|
        EnemyEntity::new_and_init(base, Speeds::new(100.0, 150.0),
        Vector2::default(), Vector2::default(), EnemySMType::Basic(EnemyStateMachine::new()))))]
    ent: OnReady<EnemyEntity>,
    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for ActualEnemyBody {
    fn physics_process(&mut self, delta: f32) {
        let mut this = self.to_gd().upcast();
        self.ent.physics_process(&mut this, delta);
    }
}
