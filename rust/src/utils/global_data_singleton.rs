use godot::prelude::*;

use crate::{
    entities::{
        entity_stats::StatModifier, movements::Direction, player::main_character::MainCharacter,
    },
    world::item::GameItem,
};

#[derive(GodotClass)]
#[class(init, base = Object)]
pub struct GlobalData {
    pub paths: PathData,
    pub player_pos: Vector2,
    pub player: Option<Gd<MainCharacter>>,
    pub player_dir: Direction,
    #[init(val = SignalHandler::new_alloc())]
    pub sigs: Gd<SignalHandler>,
    base: Base<Object>,
}

impl GlobalData {
    pub fn sig_handler(&mut self) -> __godot_Signals_SignalHandler<'_, SignalHandler> {
        self.sigs.signals()
    }

    pub fn singleton() -> Gd<Self> {
        godot::classes::Engine::singleton()
            .get_singleton(&Self::class_name().to_string_name())
            .unwrap()
            .cast::<Self>()
    }

    pub fn get_player_mut(&mut self) -> Option<&mut Gd<MainCharacter>> {
        self.player.as_mut()
    }
}

#[derive(Default)]
pub struct PathData {
    pub player: Option<String>,
    pub map: Option<String>,
}

#[derive(GodotClass)]
#[class(init, base = Object)]
pub struct SignalHandler {
    base: Base<Object>,
}

#[godot_api]
impl SignalHandler {
    #[signal]
    pub fn new_item_added(item: Gd<GameItem>);

    #[signal]
    pub fn picked_up_item(item: Gd<GameItem>);

    #[signal]
    pub fn new_modifier(modifier: Gd<StatModifier>);

    #[signal]
    pub fn modifier_removed(modifier: Gd<StatModifier>);
}
