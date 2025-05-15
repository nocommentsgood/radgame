use godot::{classes::AnimationPlayer, prelude::*};
use godot_traits::{
    enemy_state_machine,
    input_hanlder::PlatformerDirection,
    patrol_component::PatrolComponent,
    speed_component::SpeedComponent,
    stats::CharacterStats,
    timer_component::{EnemyTimers, Timer},
    traits::{
        self, animatable::Animatable, enemy_state_ext::EnemyEntityStateMachineExt,
        has_aggro::HasAggroArea, has_hitbox::HasEnemyHitbox,
    },
};

type State = enemy_state_machine::State;

const BULLET_SPEED: real = 500.0;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct ProjectileEnemy {
    velocity: Vector2,
    player_pos: Vector2,
    shoot_cooldown: Timer,
    patrol_comp: PatrolComponent,
    speeds: SpeedComponent,
    direction: PlatformerDirection,
    stats: CharacterStats,
    state: statig::blocking::StateMachine<enemy_state_machine::EnemyStateMachine>,
    timers: EnemyTimers,
    base: Base<Node2D>,
    #[init(load = "res://projectile.tscn")]
    projectile_scene: OnReady<Gd<PackedScene>>,
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
}

#[godot_api]
impl INode2D for ProjectileEnemy {
    fn ready(&mut self) {
        self.speeds = SpeedComponent::new(40.0, 40.0, 80.0);
        self.patrol_comp = PatrolComponent::new(138.0, 0.0, -138.0, 0.0);
        self.connect_aggro_area_signal();
        self.connect_hitbox_signal();
        self.timers = EnemyTimers::new(1.8, 2.0, 1.0, 2.0, 2.0);
        self.stats.health = 20;
        self.shoot_cooldown = Timer::new(2.0);
    }

    fn process(&mut self, _delta: f64) {
        match self.state.state() {
            State::Idle {} => self.idle(),
            State::Attack2 {} => {
                self.chain_attack();
            }
            State::ChasePlayer {} => self.chase_player(),
            State::Patrol {} => self.patrol(),
            State::Falling {} => (),
            State::Attack {} => {
                self.attack();
            }
        }
        self.update_timers();
    }
}

#[godot_api]
impl ProjectileEnemy {
    fn shoot_projectile(&mut self, target: Vector2) {
        let position = self.base().get_global_position();
        let mut bullet = self
            .projectile_scene
            .instantiate_as::<super::projectile::Projectile>();
        let target = position.direction_to(target).normalized_or_zero();
        bullet.bind_mut().velocity = target * BULLET_SPEED;
        self.base_mut()
            .call_deferred("add_child", &[bullet.to_variant()]);
    }

    fn attack(&mut self) {
        let target = self.get_player_pos();
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_process_delta_time() as f32;
        let attack_cooldown = self.timers.attack_cooldown.clone();
        if attack_cooldown.value == attack_cooldown.initial_value() {
            self.shoot_projectile(target);
            self.timers.attack_cooldown.value -= delta;
        }
        self.timers.attack_cooldown.value -= delta;
        self.timers.attack_animation.value -= delta;

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self) {
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_process_delta_time() as f32;
        self.timers.chain_attack.value -= delta;

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn update_timers(&mut self) {
        let delta = self.base().get_process_delta_time() as f32;

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
impl traits::character_resources::CharacterResources for ProjectileEnemy {
    fn get_health(&self) -> u32 {
        self.stats.health
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.health = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.energy
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.energy = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.mana
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.mana = amount;
    }
}

#[godot_dyn]
impl traits::damageable::Damageable for ProjectileEnemy {
    fn destroy(&mut self) {
        self.base_mut().queue_free();
    }
}

impl traits::has_state::HasState for ProjectileEnemy {
    fn sm_mut(
        &mut self,
    ) -> &mut statig::prelude::StateMachine<enemy_state_machine::EnemyStateMachine> {
        &mut self.state
    }

    fn sm(&self) -> &statig::prelude::StateMachine<enemy_state_machine::EnemyStateMachine> {
        &self.state
    }
}

impl traits::has_aggro::HasAggroArea for ProjectileEnemy {
    fn set_player_pos(&mut self, player_pos: Vector2) {
        self.player_pos = player_pos;
    }
    fn get_player_pos(&self) -> Vector2 {
        self.player_pos
    }
}

impl traits::has_hitbox::HasEnemyHitbox for ProjectileEnemy {}

impl traits::moveable::MoveableEntity for ProjectileEnemy {}

impl traits::animatable::Animatable for ProjectileEnemy {
    fn get_anim_player(&self) -> Gd<godot::classes::AnimationPlayer> {
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

impl traits::enemy_state_ext::EnemyEntityStateMachineExt for ProjectileEnemy {
    fn timers(&mut self) -> &mut EnemyTimers {
        &mut self.timers
    }
    fn attack(&mut self) {
        let target = self.player_pos;
        let time = self.timers.attack_animation.value;
        let delta = self.base().get_process_delta_time() as f32;
        let attack_cooldown = self.timers.attack_cooldown.clone();
        self.update_animation();
        if attack_cooldown.value == attack_cooldown.initial_value() {
            self.shoot_projectile(target);
            self.timers.attack_cooldown.value -= delta;
        }
        self.timers.attack_cooldown.value -= delta;
        self.timers.attack_animation.value -= delta;

        if time <= 0.0 {
            self.timers.attack_animation.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn chain_attack(&mut self) {
        let time = self.timers.chain_attack.value;
        let delta = self.base().get_process_delta_time() as f32;
        self.timers.chain_attack.value -= delta;

        if time <= 0.0 {
            self.timers.chain_attack.reset();
            self.state
                .handle(&enemy_state_machine::EnemyEvent::TimerElapsed);
        }
    }

    fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    fn speeds(&self) -> godot_traits::speed_component::SpeedComponent {
        self.speeds.clone()
    }

    fn patrol_comp(&self) -> PatrolComponent {
        self.patrol_comp.clone()
    }
}
