use godot::{
    classes::{AnimationPlayer, CharacterBody2D, ICharacterBody2D},
    obj::WithBaseField,
    prelude::*,
};
use godot_traits::{
    enemy_state_machine,
    input_hanlder::PlatformerDirection,
    patrol_component::PatrolComponent,
    speed_component::SpeedComponent,
    timer_component::EnemyTimers,
    traits::{
        self, character_resources::CharacterResources,
        enemy_state_ext::EnemyCharacterStateMachineExt, has_aggro::HasAggroArea,
        has_hitbox::HasEnemyHitbox,
    },
};

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct TestEnemy {
    direction: PlatformerDirection,
    velocity: Vector2,
    timers: EnemyTimers,
    patrol_comp: PatrolComponent,
    speeds: SpeedComponent,
    player_pos: Vector2,
    state: statig::blocking::StateMachine<enemy_state_machine::EnemyStateMachine>,
    base: Base<CharacterBody2D>,
    energy: u32,
    mana: u32,
    #[init(val = 100)]
    health: u32,
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
}

#[godot_api]
impl ICharacterBody2D for TestEnemy {
    fn ready(&mut self) {
        self.speeds = SpeedComponent::new(40.0, 40.0, 80.0);
        self.patrol_comp = PatrolComponent::new(50.0, 0.0, -50.0, 0.0);
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();
        self.timers = EnemyTimers::new(1.8, 2.0, 2.7, 2.0, 4.0);
        self.create_timer();
    }

    fn physics_process(&mut self, _delta: f64) {
        self.check_floor();
        dbg!(&self.state.state());

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

        // Update attack cooldown timer
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value < attack_cooldown.initial_value() && attack_cooldown.value > 0.0 {
            self.timers.attack_cooldown.value -= delta;
        } else if attack_cooldown.value <= 0.0 {
            self.timers.attack_cooldown.reset();
        }
    }
}

#[godot_dyn]
impl godot_traits::traits::character_resources::CharacterResources for TestEnemy {
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

impl traits::has_hitbox::HasEnemyHitbox for TestEnemy {}

impl traits::has_state::HasState for TestEnemy {
    fn sm_mut(
        &mut self,
    ) -> &mut statig::prelude::StateMachine<enemy_state_machine::EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<enemy_state_machine::EnemyStateMachine> {
        &self.state
    }
}

impl traits::has_aggro::HasAggroArea for TestEnemy {
    fn set_player_pos(&mut self, player_pos: Vector2) {
        self.player_pos = player_pos;
    }

    fn get_player_pos(&self) -> Vector2 {
        self.player_pos
    }
}

#[godot_dyn]
impl traits::damageable::Damageable for TestEnemy {
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

impl traits::animatable::Animatable for TestEnemy {
    fn get_anim_player(&self) -> Gd<AnimationPlayer> {
        self.animation_player.clone()
    }

    fn get_direction(&self) -> PlatformerDirection {
        self.direction.clone()
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }
}

impl traits::moveable::MoveableCharacter for TestEnemy {}

impl traits::enemy_state_ext::EnemyCharacterStateMachineExt for TestEnemy {
    fn timers(&mut self) -> &mut EnemyTimers {
        &mut self.timers
    }

    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    fn speeds(&self) -> SpeedComponent {
        self.speeds.clone()
    }

    fn patrol_comp(&self) -> PatrolComponent {
        self.patrol_comp.clone()
    }
}
