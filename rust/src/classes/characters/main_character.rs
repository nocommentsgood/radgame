use std::collections::HashMap;

use godot::{
    classes::{
        AnimationPlayer, Area2D, CharacterBody2D, CollisionObject2D, ICharacterBody2D, Input,
        RayCast2D, Timer,
    },
    obj::WithBaseField,
    prelude::*,
};

use crate::{
    classes::{
        characters::{
            entity_hitbox::EntityHitbox,
            shaky_player_camera::{ShakyPlayerCamera, TraumaLevel},
        },
        components::{hurtbox::Hurtbox, timer_component::Time},
        enemies::projectile::Projectile,
    },
    components::{
        managers::{
            input_hanlder::InputHandler, item::StatModifier, item_component::ItemComponent,
        },
        state_machines::{
            character_state_machine::{self, *},
            movements::PlatformerDirection,
        },
    },
    traits::components::character_components::{
        character_resources::CharacterResources, damageable::Damageable, damaging::Damaging,
        player::Player,
    },
};

use super::character_stats::{StatVal, Stats, Stats::*};
use crate::classes::components::timer_component::{PlayerTimer, Timers};

type PT = PlayerTimer;
type Event = crate::components::state_machines::character_state_machine::Event;
const GRAVITY: f32 = 980.0;

#[derive(GodotClass)]
#[class(init, base=CharacterBody2D)]
pub struct MainCharacter {
    direction: PlatformerDirection,
    velocity: Vector2,
    active_velocity: Vector2,
    hit_enemy: bool,
    can_attack_chain: bool,
    timers: Timers,
    state: statig::blocking::StateMachine<CharacterStateMachine>,
    stats: HashMap<Stats, StatVal>,
    base: Base<CharacterBody2D>,

    #[init(node = "ItemComponent")]
    pub item_comp: OnReady<Gd<ItemComponent>>,

    #[var]
    #[init(node = "DodgingCooldownTimer")]
    dodging_cooldown_timer: OnReady<Gd<Timer>>,
    #[var]
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,

    #[var]
    #[init(node = "LedgeSensor")]
    ledge_sensor: OnReady<Gd<RayCast2D>>,
}

#[godot_api]
impl ICharacterBody2D for MainCharacter {
    fn ready(&mut self) {
        let this = self.to_gd();

        let hitbox = self.base().get_node_as::<Area2D>("Hitbox");
        hitbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hitbox);
        self.item_comp
            .signals()
            .new_modifier()
            .connect_other(&this, Self::on_new_modifier);

        self.item_comp
            .signals()
            .modifier_removed()
            .connect_other(&this, Self::on_modifier_removed);

        let hurtbox = self.base().get_node_as::<Area2D>("Hurtbox");
        hurtbox
            .signals()
            .area_entered()
            .connect_other(&this, Self::on_area_entered_hurtbox);

        // TODO: Find how to get tracks for specific animations.
        // That way we can dynamically divide by scaling speed.

        // Dodging animations, independent of cardinal direction, are all of the same length.
        // Therefore, it is acceptable to use the length of any dodging animation.
        // East was arbitrarily chosen.
        let dodge_animation_length = self
            .get_animation_player()
            .get_animation("dodge_east")
            .unwrap()
            .get_length()
            / 1.5;

        let attack_animation_length = self
            .get_animation_player()
            .get_animation("attack_1_east")
            .unwrap()
            .get_length();

        let jumping_animation_length = 0.1;

        let healing_animation_length = self
            .get_animation_player()
            .get_animation("heal_east")
            .unwrap()
            .get_length();

        let parry_animation_length = self
            .get_animation_player()
            .get_animation("parry_east")
            .unwrap()
            .get_length();

        let hurt_animation_length = self
            .get_animation_player()
            .get_animation("hurt_east")
            .unwrap()
            .get_length();

        self.stats.insert(Stats::Health, StatVal::new(50));
        self.stats.insert(Stats::MaxHealth, StatVal::new(50));
        self.stats.insert(Stats::HealAmount, StatVal::new(10));
        self.stats.insert(Stats::AttackDamage, StatVal::new(30));
        self.stats.insert(Stats::RunningSpeed, StatVal::new(150));
        self.stats.insert(Stats::JumpingSpeed, StatVal::new(350));
        self.stats.insert(Stats::DodgingSpeed, StatVal::new(250));
        self.stats.insert(Stats::AttackingSpeed, StatVal::new(10));

        self.timers.0.push(Time::new(0.3));
        self.timers.0.push(Time::new(dodge_animation_length));
        self.timers.0.push(Time::new(jumping_animation_length));
        self.timers.0.push(Time::new(attack_animation_length));
        self.timers.0.push(Time::new(attack_animation_length));
        self.timers.0.push(Time::new(healing_animation_length));
        self.timers.0.push(Time::new(parry_animation_length));
        self.timers.0.push(Time::new(hurt_animation_length));
        self.timers.0.push(Time::new(0.3));
        self.timers.0.push(Time::new(0.15));

        let mut hurtbox = self.base().get_node_as::<Hurtbox>("Hurtbox");
        hurtbox.bind_mut().attack_damage = self.stats.get(&Stats::AttackDamage).unwrap().0;
    }

    fn unhandled_input(&mut self, input: Gd<godot::classes::InputEvent>) {
        if input.is_action_pressed("attack") {
            self.state.handle(&Event::AttackButton);
        }
        if input.is_action_pressed("jump") {
            self.state.handle(&Event::JumpButton);
        }
        if input.is_action_released("jump") {
            self.state.handle(&Event::ActionReleasedEarly);
        }
        if input.is_action_pressed("dodge") {
            self.state.handle(&Event::DodgeButton);
        }
        if input.is_action_pressed("heal") {
            self.state.handle(&Event::HealingButton);
        }
        if input.is_action_pressed("parry") {
            self.state.handle(&Event::ParryButton);
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        let input = Input::singleton();
        let event = InputHandler::to_platformer_event(&Input::singleton());
        self.velocity.x = InputHandler::get_velocity(&input).x;

        if !self.base().is_on_floor() {
            self.state.handle(&Event::FailedFloorCheck);
        }

        match self.state.state() {
            character_state_machine::State::Attacking {} => self.attack(),
            character_state_machine::State::Attack2 {} => self.attack_2(),
            character_state_machine::State::Parry {} => self.parry(),
            character_state_machine::State::Idle {} => self.idle(),
            character_state_machine::State::Dodging {} => self.dodge(),
            character_state_machine::State::Jumping {} => self.jump(),
            character_state_machine::State::Falling {} => self.fall(),
            character_state_machine::State::Moving {} => self.move_character(),
            character_state_machine::State::Healing {} => self.heal(),
            character_state_machine::State::Grappling {} => self.grapple(),
            character_state_machine::State::Hurt {} => self.hurt(),
        }
        self.state.handle(&event);
    }
}

#[godot_api]
impl MainCharacter {
    #[signal]
    pub fn player_health_changed(previous_health: u32, new_health: u32, damage_amount: u32);

    #[signal]
    fn parried_attack();

    #[signal]
    fn player_died();

    // Had to resort to enabling and disabling the collision shape manually, otherwise the
    // `area_entered()` signal of the `Hurtbox` would emit twice.
    fn on_area_entered_hurtbox(&mut self, area: Gd<Area2D>) {
        if let Ok(_hurtbox) = area.try_cast::<EntityHitbox>() {
            dbg!("can attack chain");
            self.hit_enemy = true;
            self.base()
                .get_node_as::<godot::classes::CollisionShape2D>("Hurtbox/HurtboxShape")
                .set_deferred("disabled", &true.to_variant());
        }
    }

    fn on_area_entered_hitbox(&mut self, area: Gd<Area2D>) {
        if !self.parried_attack(area.clone()) {
            let damaging = DynGd::<Area2D, dyn Damaging>::from_godot(area);
            let target = self.to_gd().upcast::<Node2D>();
            let guard = self.base_mut();
            let damageable = DynGd::<Node2D, dyn Damageable>::from_godot(target);
            damaging.dyn_bind().do_damage(damageable);
            drop(guard);
            let mut camera = self
                .base()
                .get_node_as::<ShakyPlayerCamera>("ShakyPlayerCamera");
            camera
                .bind_mut()
                .add_trauma(TraumaLevel::from(damaging.dyn_bind().damage_amount()));
            self.state.handle(&Event::Hurt);
        }
    }

    fn detect_ledges(&mut self) {
        if self.get_ledge_sensor().is_colliding() {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.state.handle(&Event::GrabbedLedge);
            if let Some(obj) = self.get_ledge_sensor().get_collider() {
                let collision = obj.cast::<CollisionObject2D>();
                let shape_id = self.get_ledge_sensor().get_collider_shape();
                let owner = collision.shape_find_owner(shape_id);
                let shape = collision.shape_owner_get_owner(owner);
                let s = shape.unwrap().cast::<godot::classes::CollisionShape2D>();
                dbg!(&s.get_shape().unwrap().get_rect());
            }
        }
    }

    // TODO: Ledge grappling is buggy.
    fn grapple(&mut self) {
        let input = Input::singleton();
        self.base_mut().set_velocity(Vector2::ZERO);
        self.update_animation();

        if input.is_action_just_pressed("west") & self.get_ledge_sensor().is_colliding()
            || input.is_action_just_pressed("east") & self.get_ledge_sensor().is_colliding()
        {
            self.state.handle(&Event::WasdJustPressed);
        }
    }

    fn dodge(&mut self) {
        let delta = self.base().get_physics_process_delta_time() as f32;
        let mut cooldown_timer = self.get_dodging_cooldown_timer();
        let time = self.timers.get(&PT::DodgeAnimation);

        if cooldown_timer.get_time_left() > 0.0 {
            self.state.handle(&Event::TimerInProgress);
        } else if time < self.timers.get_init(&PT::DodgeAnimation) && time > 0.0 {
            self.base_mut().move_and_slide();
            self.timers.set(&PT::DodgeAnimation, time - delta);
        } else {
            let speed = self.stats.get(&DodgingSpeed).unwrap().0 as f32;
            let velocity = self.velocity;

            self.base_mut().set_velocity(velocity * speed);
            self.base_mut().move_and_slide();
            self.update_animation();
            self.timers.set(&PT::DodgeAnimation, time - delta);

            if time <= 0.0 {
                self.timers.reset(&PT::DodgeAnimation);
                self.state.handle(&Event::TimerElapsed);
                cooldown_timer.start();
            }
        }
    }

    fn attack(&mut self) {
        // TODO: Maybe there is a better way of ignoring input. If the player is hit during an
        // attack, the state machine switches to 'hurt' state (as it should), but input handling is
        // never turned back on.
        self.base_mut().set_process_unhandled_input(false);
        let time = self.timers.get(&PT::AttackAnimation);
        let ac_timer = self.timers.get(&PT::AttackChain);
        let delta = self.base().get_physics_process_delta_time() as f32;
        let mut h_shape = self
            .base()
            .get_node_as::<godot::classes::CollisionShape2D>("Hurtbox/HurtboxShape");

        if time < self.timers.get_init(&PT::AttackAnimation) && time > 0.0 {
            h_shape.set_deferred("disabled", &true.to_variant());
            self.timers.set(&PT::AttackAnimation, time - delta);

            if Input::singleton().is_action_pressed("parry") {
                self.timers.reset(&PT::AttackAnimation);
                self.base_mut().set_process_unhandled_input(true);
                self.state.handle(&Event::ParryButton);
            }

            if ac_timer < self.timers.get_init(&PT::AttackChain) && ac_timer > 0.0 {
                if Input::singleton().is_action_just_pressed("attack") {
                    if self.hit_enemy {
                        self.can_attack_chain = true;
                        self.hit_enemy = false;
                    }
                } else {
                    self.timers.set(&PT::AttackChain, ac_timer - delta);
                }
            }
        } else {
            h_shape.set_deferred("disabled", &false.to_variant());
            self.update_direction();
            self.update_animation();
            self.timers.set(&PT::AttackAnimation, time - delta);
            self.timers.set(&PT::AttackChain, ac_timer - delta);
        }
        if time <= 0.0 {
            self.base_mut().set_process_unhandled_input(true);
            h_shape.set_deferred("disabled", &true.to_variant());
            self.timers.reset(&PT::AttackAnimation);
            self.timers.reset(&PT::AttackChain);
            if self.can_attack_chain {
                self.can_attack_chain = false;
                self.hit_enemy = false;
                self.state.handle(&Event::AttackButton);
            } else {
                self.hit_enemy = false;
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn attack_2(&mut self) {
        self.can_attack_chain = false;
        let x = PT::AttackAnimation2;
        let time = self.timers.get(&x);
        let delta = self.base().get_physics_process_delta_time() as f32;

        if time < self.timers.get_init(&x) && time > 0.0 {
            self.timers.set(&x, time - delta);
        } else {
            self.update_animation();
            self.timers.set(&x, time - delta);

            if time <= 0.0 {
                self.timers.reset(&x);
                self.state.handle(&Event::TimerElapsed);
            }
        }
    }

    fn hurt(&mut self) {
        self.base_mut().set_process_unhandled_input(true);
        let time = self.timers.get(&PT::HurtAnimation);
        let delta = self.base().get_physics_process_delta_time();
        if time > 0.0 {
            self.timers.set(&PT::HurtAnimation, time - delta as f32);
            self.base_mut().set_velocity(Vector2::ZERO);
            self.update_animation();
        }

        if time <= 0.0 {
            self.timers.reset(&PT::HurtAnimation);
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn idle(&mut self) {
        self.active_velocity = Vector2::ZERO;
        self.update_animation();
    }

    fn move_character(&mut self) {
        let target_velocity = self.velocity * self.stats.get(&RunningSpeed).unwrap().0 as f32;
        self.active_velocity = self.active_velocity.lerp(target_velocity, 0.2);
        let velocity = self.active_velocity;

        self.update_direction();
        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }

    fn jump(&mut self) {
        let x = PT::JumpingAnimation;
        let time = self.timers.get(&x);
        let delta = self.base().get_physics_process_delta_time() as f32;

        if self.base().is_on_floor() {
            self.velocity.y = Vector2::UP.y * self.stats.get(&JumpingSpeed).unwrap().0 as f32;
            self.velocity.x *= self.stats.get(&RunningSpeed).unwrap().0 as f32;
            let velocity = self.velocity;
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else {
            self.velocity.y += GRAVITY * delta;
            let target_x = self.velocity.x * self.stats.get(&RunningSpeed).unwrap().0 as f32;
            self.active_velocity.x = self.active_velocity.x.lerp(target_x, 0.2);
            let velocity = Vector2::new(self.active_velocity.x, self.velocity.y);
            self.update_direction();
            self.update_animation();
            self.detect_ledges();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
            self.timers.set(&x, time - delta);
        }

        if time <= 0.0 {
            self.timers.reset(&x);
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn heal(&mut self) {
        let x = PT::HealingAnimation;
        let time = self.timers.get(&x);
        let current_health = self.stats.get(&Stats::Health).unwrap().0;
        let amount = self.stats.get(&Stats::HealAmount).unwrap().0;
        let max = self.stats.get(&Stats::MaxHealth).unwrap().0;
        let delta = self.base().get_physics_process_delta_time() as f32;
        self.velocity = Vector2::ZERO;
        let velocity = self.velocity;

        self.update_animation();
        self.base_mut().set_velocity(velocity);
        self.timers.set(&x, time - delta);

        if time <= 0.0 {
            if current_health < max {
                self.stats.get_mut(&Stats::Health).unwrap().0 += amount;
                let new = self.stats.get(&Stats::Health).unwrap().0;
                self.signals()
                    .player_health_changed()
                    .emit(current_health, new, amount);
            }
            self.timers.reset(&x);
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn fall(&mut self) {
        let x = &PT::JumpingAnimation;
        if !self.base().is_on_floor() {
            let delta = self.base().get_physics_process_delta_time() as f32;
            self.velocity.y += GRAVITY * delta;
            self.velocity.x *= self.stats.get(&RunningSpeed).unwrap().0 as f32;

            let velocity = self.velocity;
            self.update_direction();
            self.update_animation();
            self.detect_ledges();
            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();
        } else if self.base().is_on_floor() {
            self.velocity.y = 0.0;
            self.state.handle(&Event::OnFloor);
            if self.timers.get(x) < self.timers.get_init(x) {
                self.timers.reset(x);
            }
        }
    }

    fn parry(&mut self) {
        let anim_time = self.timers.get(&PT::ParryAnimation);
        let parry_time = self.timers.get(&PT::Parry);
        let perf_parry = self.timers.get(&PT::PerfectParry);
        let delta = self.base().get_physics_process_delta_time() as f32;
        self.update_animation();
        self.timers.set(&PT::ParryAnimation, anim_time - delta);
        self.timers.set(&PT::Parry, parry_time - delta);
        self.timers.set(&PT::PerfectParry, perf_parry - delta);

        if anim_time <= 0.0 {
            self.timers.reset(&PT::ParryAnimation);
            self.timers.reset(&PT::Parry);
            self.timers.reset(&PT::PerfectParry);
            self.state.handle(&Event::TimerElapsed);
        }
    }

    fn parried_attack(&mut self, area: Gd<Area2D>) -> bool {
        match self.state.state() {
            State::Parry {} => {
                if self.timers.get(&PT::PerfectParry) > 0.0 {
                    println!("\nPERFECT PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    self.timers.reset(&PT::PerfectParry);
                    true
                } else if self.timers.get(&PT::Parry) > 0.0 {
                    println!("\nNORMAL PARRY\n");
                    if area.is_in_group("enemy_projectile")
                        && let Some(parent) = area.get_parent()
                        && let Ok(mut projectile) = parent.try_cast::<Projectile>()
                    {
                        projectile.bind_mut().on_parried();
                    }
                    self.signals().parried_attack().emit();
                    self.timers.reset(&PT::Parry);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn get_current_animation(&self) -> String {
        let mut animation = self.state.state().to_string();
        animation.push('_');
        animation.push_str(self.direction.to_string().as_str());

        animation
    }

    fn update_animation(&mut self) {
        let animation = self.get_current_animation();
        self.animation_player.play_ex().name(&animation).done();
        self.animation_player.advance(0.0);
    }

    fn update_direction(&mut self) {
        if !self.velocity.x.is_zero_approx() {
            self.direction = PlatformerDirection::from_platformer_velocity(&self.velocity);
        }
    }

    fn on_new_modifier(&mut self, modifier: Gd<StatModifier>) {
        if let Some(val) = self.stats.get_mut(&modifier.bind().stat) {
            val.apply_modifier(modifier.bind().clone());
        }
    }

    fn on_modifier_removed(&mut self, modifier: Gd<StatModifier>) {
        if let Some(val) = self.stats.get_mut(&modifier.bind().stat) {
            val.remove_modifier(modifier.bind().clone());
        }
    }
}

#[godot_dyn]
impl CharacterResources for MainCharacter {
    fn get_health(&self) -> u32 {
        self.stats.get(&Stats::Health).unwrap().0
    }

    fn set_health(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Health).unwrap().0 = amount;
    }

    fn get_energy(&self) -> u32 {
        self.stats.get(&Stats::Energy).unwrap().0
    }

    fn set_energy(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Energy).unwrap().0 = amount;
    }

    fn get_mana(&self) -> u32 {
        self.stats.get(&Stats::Mana).unwrap().0
    }

    fn set_mana(&mut self, amount: u32) {
        self.stats.get_mut(&Stats::Mana).unwrap().0 = amount;
    }
}

#[godot_dyn]
impl Damageable for MainCharacter {
    fn take_damage(&mut self, amount: u32) {
        let previous_health = self.get_health();
        let current_health = previous_health.saturating_sub(amount);
        println!("previous health {previous_health} ... current health {current_health}");

        self.set_health(current_health);
        self.signals()
            .player_health_changed()
            .emit(previous_health, current_health, amount);

        if self.is_dead() {
            println!("You died");
            self.destroy();
        }
    }

    fn destroy(&mut self) {
        self.signals().player_died().emit();
        self.base_mut().queue_free();
    }
}

#[godot_dyn]
impl Damaging for MainCharacter {
    fn damage_amount(&self) -> u32 {
        self.stats.get(&AttackDamage).unwrap().0
    }
}

#[godot_dyn]
impl Player for MainCharacter {}
