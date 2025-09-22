use std::collections::VecDeque;

use godot::{
    builtin::Vector2,
    classes::{IStaticBody2D, Node2D, PackedScene, StaticBody2D, Timer},
    obj::{Base, Gd, NewAlloc, OnReady, WithBaseField, WithDeferredCall},
    prelude::{GodotClass, godot_api},
    tools::load,
};

use crate::entities::{
    damage::{self, AttackData, Damage, DamageType, ElementType},
    entity_hitbox::Hurtbox,
    player::main_character::MainCharacter,
};

#[derive(Clone, Copy, Debug)]
pub enum Ability {
    JumpPlatform,
    TwinPillar,
}
impl Ability {
    pub fn execute(&self, player: &mut MainCharacter) {
        match self {
            Ability::JumpPlatform => {
                let mut plat =
                    load::<PackedScene>("uid://cul64aw83q0sm").instantiate_as::<JumpPlatform>();
                let pos = player.base().get_position();
                let dir = player.get_direction();
                match dir {
                    crate::entities::movements::Direction::Right => {
                        plat.set_constant_linear_velocity(Vector2::new(100.0, 0.0));
                        plat.set_position(pos + Vector2::new(40.0, 0.0));
                        plat.bind_mut().target = pos + Vector2::new(100.0, 0.0);
                        plat.bind_mut().velocity = pos.direction_to(pos + Vector2::new(100.0, 0.0));
                    }
                    crate::entities::movements::Direction::Left => {
                        plat.set_constant_linear_velocity(Vector2::new(-100.0, 0.0));
                        plat.set_position(pos + Vector2::new(-40.0, 0.0));
                        plat.bind_mut().target = pos + Vector2::new(-100.0, 0.0);
                        plat.bind_mut().velocity =
                            pos.direction_to(pos + Vector2::new(-100.0, 0.0));
                    }
                }
                plat.bind_mut().start = pos;
                player.base_mut().add_sibling(&plat);
            }
            Ability::TwinPillar => {
                let mut ability =
                    load::<PackedScene>("uid://dnfo3s5ywpq6m").instantiate_as::<Node2D>();
                let mut timer = Timer::new_alloc();

                // Set ability's damage amount.
                let mut right_pillar = ability.get_node_as::<Hurtbox>("RightPillar");
                right_pillar.bind_mut().data = Some(AttackData {
                    hurtbox: right_pillar.clone(),
                    parryable: false,
                    damage: Damage {
                        raw: 10,
                        d_type: DamageType::Elemental(ElementType::Magic),
                    },
                });
                let mut left_pillar = ability.get_node_as::<Hurtbox>("LeftPillar");
                left_pillar.bind_mut().data = Some(AttackData {
                    hurtbox: right_pillar.clone(),
                    parryable: false,
                    damage: Damage {
                        raw: 10,
                        d_type: DamageType::Elemental(ElementType::Magic),
                    },
                });

                timer.set_wait_time(1.5);
                ability.set_position(player.base().get_global_position());
                ability.add_child(&timer);
                player.base_mut().add_sibling(&ability);

                timer
                    .signals()
                    .timeout()
                    .connect(move || ability.queue_free());
                timer.start();
            }
        }
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Node2D)]
struct TwinPillarAbility {
    #[init(node = "RightPillar")]
    right: OnReady<Gd<Hurtbox>>,
    #[init(node = "LeftPillar")]
    left: OnReady<Gd<Hurtbox>>,
    base: Base<Node2D>,
}

#[derive(GodotClass, Debug)]
#[class(init, base=StaticBody2D)]
struct JumpPlatform {
    pub velocity: Vector2,
    pub target: Vector2,
    pub start: Vector2,
    collision_count: u32,
    #[init(node = "FreeTimer")]
    free_timer: OnReady<Gd<Timer>>,
    #[init(node = "ChangeTimer")]
    change_timer: OnReady<Gd<Timer>>,
    base: Base<StaticBody2D>,
}

#[godot_api]
impl IStaticBody2D for JumpPlatform {
    fn ready(&mut self) {
        self.free_timer.set_wait_time(4.0);
        self.free_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::free);

        self.change_timer.set_wait_time(2.0);
        self.change_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::change_dir);

        self.free_timer.start();
        self.change_timer.start();
        self.start = self.base().get_position();
    }

    fn physics_process(&mut self, delta: f32) {
        let velocity = self.velocity * 100.0;

        let kin = self.base_mut().move_and_collide(velocity * delta);
        if let Some(col) = kin
            && let Some(obj) = col.get_collider()
            && !obj.is_class("MainCharacter")
        {
            self.change_dir();
        }
    }
}

#[godot_api]
impl JumpPlatform {
    fn free(&mut self) {
        self.apply_deferred(|this| this.base_mut().queue_free());
    }

    fn change_dir(&mut self) {
        self.collision_count += 1;
        if self.collision_count == 2 {
            self.free();
        }
        self.change_timer.stop();
        let cur_pos = self.base().get_position();
        self.velocity = cur_pos.direction_to(self.start);
        let lin_vel = self.base().get_constant_linear_velocity() * -1.0;
        self.base_mut().set_constant_linear_velocity(lin_vel);
    }
}

#[derive(Default, Clone, Debug)]
pub struct AbilityComp {
    /// The player's quick abilities. Limited to a capacity of 3 abilities.
    pub quick: VecDeque<Option<Ability>>,
}

impl AbilityComp {
    #[allow(unused)]
    pub fn new() -> Self {
        let mut this = Self {
            quick: VecDeque::with_capacity(3),
        };
        for _ in 0..2 {
            this.quick.push_back(None);
        }
        this
    }

    pub fn new_test() -> Self {
        let mut this = Self {
            quick: VecDeque::with_capacity(3),
        };
        this.quick.push_back(Some(Ability::JumpPlatform));
        this.quick.push_back(Some(Ability::TwinPillar));
        this.quick.push_back(None);
        this
    }
}
