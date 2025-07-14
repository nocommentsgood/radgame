use godot::prelude::*;

use crate::{
    classes::scenes::{closing_door::ClosingDoor, environment_trigger::EnvironmentTrigger},
    components::managers::enemy_spawner::EnemySpawner,
};

#[derive(GodotClass)]
#[class(base = Node, init)]
struct ArenaScene {
    // proj_enemy_spawner: Gd<EnemySpawner>,
    #[init(node = "World")]
    world: OnReady<Gd<Node>>,

    #[init(load = "uid://bcae4wnfye0do")]
    proj_enemy: OnReady<Gd<PackedScene>>,

    #[init(node = "World/EnemySpawner")]
    spawner: OnReady<Gd<EnemySpawner>>,

    base: Base<Node>,
}

#[godot_api]
impl INode for ArenaScene {
    fn ready(&mut self) {
        let door = self
            .base()
            .get_node_as::<EnvironmentTrigger>("World/EnvironmentTrigger");
        door.signals()
            .prepare_arena()
            .connect_other(&self.to_gd(), Self::on_player_entered_door);
    }
}

#[godot_api]
impl ArenaScene {
    fn on_player_entered_door(&mut self) {
        let mut door = self.base().get_node_as::<ClosingDoor>("World/SlidingDoor");
        door.bind_mut().close();

        let e = self.spawner.bind_mut().spawn();
        let timer = self
            .base()
            .get_tree()
            .unwrap()
            .create_timer_ex(3.0)
            .process_always(true)
            .done()
            .unwrap();
        let mut this = self.to_gd();
        timer
            .signals()
            .timeout()
            .connect(move || this.bind_mut().world.add_child(&e));
    }
}
