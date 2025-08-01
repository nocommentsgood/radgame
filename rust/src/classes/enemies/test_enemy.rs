use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D},
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    classes::{
        components::{
            speed_component::SpeedComponent,
            timer_component::{EnemyTimer, Time, Timers},
        },
        enemies::patrol_component::PatrolComp,
    },
    components::state_machines::{
        enemy_state_machine::{self, *},
        movements::Direction,
    },
    traits::components::character_components::{
        self, animatable::Animatable, character_resources::CharacterResources,
        enemy_state_ext::EnemyCharacterStateMachineExt, has_aggro::HasAggroArea,
        has_hitbox::HasEnemyHitbox, has_state::HasState, moveable::MoveableCharacter,
    },
};

type ET = EnemyTimer;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    direction: Direction,
    velocity: Vector2,
    timers: Timers,
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
        self.speeds = SpeedComponent::new(40.0, 40.0, 80.0);
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();
        self.timers.0.push(Time::new(1.8));
        self.timers.0.push(Time::new(2.0));
        self.timers.0.push(Time::new(2.7));
        self.timers.0.push(Time::new(2.0));
        self.timers.0.push(Time::new(4.0));
        self.base()
            .get_node_as::<crate::classes::components::hurtbox::Hurtbox>("EnemySensors/Hurtboxes")
            .bind_mut()
            .attack_damage = 10;
    }

    fn physics_process(&mut self, _delta: f64) {
        self.check_floor();
        // dbg!(&self.state.state());

        match self.state.state() {
            enemy_state_machine::State::Idle {} => self.idle(),
            enemy_state_machine::State::ChasePlayer {} => self.chase_player(),
            enemy_state_machine::State::Patrol {} => self.patrol(),
            enemy_state_machine::State::Attack {} => self.attack(),
            enemy_state_machine::State::Attack2 {} => self.chain_attack(),
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

    // Leaving this somewhat open ended in case more timers are added later
    fn update_timers(&mut self) {
        let delta = self.base().get_physics_process_delta_time() as f32;
        let ac = &ET::AttackCooldown;

        // Update attack cooldown timer
        let attack_cooldown = self.timers.get(ac);
        if attack_cooldown < self.timers.get_init(ac) && attack_cooldown > 0.0 {
            self.timers.set(ac, attack_cooldown - delta);
        } else if attack_cooldown <= 0.0 {
            self.timers.reset(ac);
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

impl HasEnemyHitbox for TestEnemy {}

impl HasState for TestEnemy {
    fn sm_mut(&mut self) -> &mut statig::prelude::StateMachine<EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<EnemyStateMachine> {
        &self.state
    }
}

impl HasAggroArea for TestEnemy {
    fn set_player_pos(&mut self, pos: Option<Vector2>) {
        self.player_pos = pos;
    }
}

#[godot_dyn]
impl character_components::damageable::Damageable for TestEnemy {
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
    fn timers(&mut self) -> &mut crate::classes::components::timer_component::Timers {
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
