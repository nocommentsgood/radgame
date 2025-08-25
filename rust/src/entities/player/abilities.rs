use godot::{
    builtin::Vector2,
    classes::{Area2D, IStaticBody2D, Node2D, PackedScene, StaticBody2D, Timer},
    obj::{Base, Gd, NewAlloc, OnReady, WithBaseField, WithDeferredCall},
    prelude::{GodotClass, godot_api},
    tools::load,
};

use crate::entities::player::main_character::MainCharacter;

pub enum AbilityType<'a> {
    JumpPlatform(&'a mut MainCharacter),
    TwinPillar(&'a mut MainCharacter),
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
        // timers
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
    }
}

pub fn spawn_jump_platform(entity: &mut MainCharacter) {
    let mut plat = load::<PackedScene>("uid://cul64aw83q0sm").instantiate_as::<JumpPlatform>();
    let pos = entity.base().get_position();
    let dir = entity.get_direction();
    match dir {
        crate::entities::movements::Direction::East => {
            plat.set_position(pos + Vector2::new(40.0, 0.0));
            plat.bind_mut().target = pos + Vector2::new(100.0, 0.0);
            plat.bind_mut().velocity = pos.direction_to(pos + Vector2::new(100.0, 0.0));
        }
        crate::entities::movements::Direction::West => {
            plat.set_position(pos + Vector2::new(-40.0, 0.0));
            plat.bind_mut().target = pos + Vector2::new(-100.0, 0.0);
            plat.bind_mut().velocity = pos.direction_to(pos + Vector2::new(-100.0, 0.0));
        }
    }
    plat.bind_mut().start = pos;
    entity.base_mut().add_sibling(&plat);
}

impl Ability for JumpPlatform {
    fn execute(&mut self, player: &mut MainCharacter) {
        todo!()
    }
}

// TODO: Add movement comp to pillars.
#[derive(Debug, Default)]
// #[class(init, base = Node)]
pub struct TwinPillarAbility {
    // free_timer: Option<Gd<Timer>>,
    // damage: u32,
    // speed: f32,
    // #[init(node = "LeftPillar")]
    // left_pillar: Gd<Area2D>,
    // #[init(node = "RightPillar")]
    // right_pillar: Gd<Area2D>,
    // base: Base<Node>,
}

impl Ability for TwinPillarAbility {
    fn execute(&mut self, player: &mut MainCharacter) {
        let mut timer = Timer::new_alloc();
        timer.set_wait_time(1.0);

        let mut ability = load::<PackedScene>("uid://dnfo3s5ywpq6m").instantiate_as::<Node2D>();
        let left_pillar = ability.get_node_as::<Area2D>("LeftPillar");
        let right_pillar = ability.get_node_as::<Area2D>("RightPillar");
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

pub trait Ability: std::fmt::Debug {
    fn execute(&mut self, player: &mut MainCharacter);
}

pub fn spawn_ability(ability: AbilityType) {
    match ability {
        AbilityType::JumpPlatform(entity) => spawn_jump_platform(entity),
        AbilityType::TwinPillar(entity) => TwinPillarAbility::default().execute(entity),
    }
}
