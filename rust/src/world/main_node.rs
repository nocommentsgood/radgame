use godot::prelude::*;

use super::map::Map;

use crate::{entities::player::main_character::MainCharacter, utils::constants};

#[derive(GodotClass)]
#[class(init, base = Node)]
struct Main {
    #[init(val = OnReady::manual())]
    map: OnReady<Gd<Map>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for Main {
    fn ready(&mut self) {
        self.map.init(
            self.base().get_node_as::<Map>(
                constants::get_world_data()
                    .bind()
                    .paths
                    .map
                    .as_ref()
                    .unwrap(),
            ),
        );
        self.map
            .signals()
            .propigate_map_trans()
            .connect_other(&self.to_gd(), Self::on_transition_map_request);
    }

    fn enter_tree(&mut self) {
        let tree = self.base().get_tree().unwrap();
        tree.signals()
            .node_added()
            .connect_other(&self.to_gd(), Self::on_player_entered_tree);
        tree.signals()
            .node_removed()
            .connect_other(&self.to_gd(), Self::on_player_exited_tree);
    }
}

#[godot_api]
impl Main {
    // Update world data when player path changes.
    fn on_player_entered_tree(&mut self, node: Gd<Node>) {
        if let Ok(player) = node.try_cast::<MainCharacter>() {
            constants::get_world_data()
                .bind_mut()
                .paths
                .player
                .replace(player.get_path().to_string());
        }
    }

    fn on_player_exited_tree(&mut self, node: Gd<Node>) {
        if let Ok(_p) = node.try_cast::<MainCharacter>() {
            constants::get_world_data().bind_mut().paths.player.take();
        }
    }

    fn on_transition_map_request(&mut self, next_map: Gd<PackedScene>) {
        let mut world = self.base().get_node_as::<Node>("World");
        let next = next_map.instantiate();
        let mut cur_map = self.map.clone();

        if let Some(scene) = next
            && let Ok(map) = scene.try_cast::<Map>()
        {
            let value = map.clone();
            world.apply_deferred(move |world| {
                world.add_child(&value);
                world.remove_child(&cur_map);
                cur_map.queue_free();
            });

            let mut player = self.base().get_node_as::<MainCharacter>(
                constants::get_world_data()
                    .bind()
                    .paths
                    .player
                    .as_ref()
                    .unwrap(),
            );
            let new_map = map.clone();
            let timer = self.base().get_tree().unwrap().create_timer(0.1).unwrap();
            godot::task::spawn(async move {
                new_map.signals().ready().to_future().await;
                let pos = new_map.bind().player_spawn_pos;

                // BUG: After moving the player the player will float until movement input. I think
                // manually stepping the physics server would fix this. See Godot issue #76462
                player.set_position(pos);

                // Wait for camera to move to player's position.
                // Additional scope used to prevent double borrow of player.
                {
                    let mut camera = player.bind().camera.clone();
                    camera.set_drag_vertical_enabled(false);
                    camera.set_drag_horizontal_enabled(false);
                    camera.set_position_smoothing_enabled(false);
                    timer.signals().timeout().to_future().await;
                    camera.set_drag_vertical_enabled(true);
                    camera.set_drag_horizontal_enabled(true);
                    camera.set_position_smoothing_enabled(true);
                }
            });

            map.signals()
                .propigate_map_trans()
                .connect_other(&self.to_gd(), Self::on_transition_map_request);
            *self.map = map;
        }
    }
}
