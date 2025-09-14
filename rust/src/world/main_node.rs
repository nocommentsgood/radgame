use godot::{
    classes::{Area2D, Camera2D, CanvasLayer, ColorRect, Marker2D},
    prelude::*,
};

use super::map::Map;

use crate::{
    entities::player::{
        item_component::ItemComponent, main_character::MainCharacter,
        shaky_player_camera::PlayerCamera,
    },
    utils::global_data_singleton::GlobalData,
    world::{
        environment_trigger::CameraData,
        item::{GameItem, GameItemSignalHandler},
    },
};

#[derive(GodotClass)]
#[class(init, base = Node)]
struct Main {
    #[init(val = OnReady::manual())]
    map: OnReady<Gd<Map>>,
    temp_holder: Option<Gd<PlayerCamera>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for Main {
    fn ready(&mut self) {
        self.map.init(
            self.base()
                .get_node_as::<Map>(GlobalData::singleton().bind().paths.map.as_ref().unwrap()),
        );

        self.map
            .signals()
            .propigate_map_trans()
            .connect_other(&self.to_gd(), Self::on_transition_map_request);

        self.signals()
            .map_ready()
            .connect_self(Self::fade_camera_in);

        let this = self.to_gd();
        if let Some(path) = &GlobalData::singleton().bind().paths.player {
            let player = self.base().get_node_as::<MainCharacter>(path);

            // Give the map's `CameraTransition` nodes a ref to the player's camera.
            let player_cam = player.get_node_as::<PlayerCamera>("ShakyPlayerCamera");
            self.map
                .bind_mut()
                .camera_data
                .iter_mut()
                .for_each(|c| c.bind_mut().player_camera = Some(player_cam.clone()));

            player_cam
                .signals()
                .request_detach()
                .connect_other(&this, Self::on_player_camera_request_detach);

            player_cam
                .signals()
                .request_attach()
                .connect_other(&this, Self::on_player_camera_request_attach);

            let mut item_comp = player.get_node_as::<ItemComponent>("ItemComponent");
            Self::connect_map_items(&mut self.map.bind_mut().items, &mut item_comp);
        }
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
    #[signal]
    fn map_ready();

    // Update world data when player path changes.
    fn on_player_entered_tree(&mut self, node: Gd<Node>) {
        if let Ok(player) = node.try_cast::<MainCharacter>() {
            GlobalData::singleton()
                .bind_mut()
                .paths
                .player
                .replace(player.get_path().to_string());
        }
    }

    fn on_player_exited_tree(&mut self, node: Gd<Node>) {
        if let Ok(_p) = node.try_cast::<MainCharacter>() {
            GlobalData::singleton().bind_mut().paths.player.take();
        }
    }

    fn on_scene_transition_request(&mut self, position: Gd<Marker2D>) {
        if let Some(path) = dbg!(&GlobalData::singleton().bind().paths.player) {
            let mut player = self.base().get_node_as::<MainCharacter>(path);
            player
                .bind_mut()
                .camera
                .set_global_position(position.get_global_position());
        }
    }

    fn on_transition_map_request(&mut self, next_map: Gd<PackedScene>) {
        let mut this = self.to_gd();
        let future = self.fade_camera_out();
        godot::task::spawn(async move {
            future.await;
            let mut world = this.get_node_as::<Node>("World");
            let next = next_map.instantiate();
            let mut cur_map = this.bind().map.clone();

            if let Some(scene) = next
                && let Ok(mut map) = scene.try_cast::<Map>()
            {
                map.set_name("Map");
                let map_clone = map.clone();

                world.apply_deferred(move |world| {
                    world.add_child(&map_clone);
                    world.remove_child(&cur_map);
                    world.move_child(&map_clone, 0);
                    cur_map.queue_free();
                });

                let mut player = this.get_node_as::<MainCharacter>(
                    GlobalData::singleton()
                        .bind()
                        .paths
                        .player
                        .as_ref()
                        .unwrap(),
                );
                let new_map = map.clone();
                let timer = this.get_tree().unwrap().create_timer(0.1).unwrap();

                new_map.signals().ready().to_future().await;
                let pos = new_map.bind().player_spawn_pos;
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

                let player = this.get_node_as::<MainCharacter>(
                    GlobalData::singleton()
                        .bind()
                        .paths
                        .player
                        .as_ref()
                        .unwrap(),
                );
                let mut new_map = map.clone();
                let mut item_comp = player.get_node_as::<ItemComponent>("ItemComponent");
                Self::connect_map_items(&mut new_map.bind_mut().items, &mut item_comp);
                map.signals()
                    .propigate_map_trans()
                    .connect_other(&this, Self::on_transition_map_request);
                *this.bind_mut().map = map;

                this.signals().map_ready().emit();
            }
        });
    }

    fn connect_map_items(map_items: &mut [Gd<GameItem>], player_item_comp: &mut Gd<ItemComponent>) {
        for item in map_items.iter_mut() {
            Self::init_game_items(item);
        }

        let mp: Vec<_> = map_items.to_vec();
        let comp = player_item_comp.clone();

        // Wait for GameItemSignalHandler to be ready.
        godot::task::spawn(async move {
            for item in mp {
                let guard = item.bind();
                let sig_handler = guard.sig_handler.as_ref().unwrap();
                let mut p_comp = comp.clone();

                sig_handler
                    .signals()
                    .player_entered_item_area()
                    .connect(move |item| p_comp.bind_mut().set_in_item_area(item));

                let mut t_comp = comp.clone();
                sig_handler
                    .signals()
                    .player_exited_item_area()
                    .connect(move || t_comp.bind_mut().set_exited_item_area());
            }
        });
    }

    /// Initializes properties of a `Gd<GameItem>`, as the `GameItem` does not store a `Base<T>`
    /// field.
    fn init_game_items(game_item: &mut Gd<GameItem>) {
        if let Ok(mut game_item) = game_item.clone().try_cast::<GameItem>() {
            let mut node = GameItemSignalHandler::new_alloc();
            node.set_name("GameItemSignalHandler");

            game_item.set_as_top_level(true);
            game_item.call_deferred("add_child", vslice![&node.to_variant()]);
            game_item.add_to_group("items");
            game_item.bind_mut().sig_handler = Some(node);

            let area = game_item.get_node_as::<Area2D>("Area2D");
            let mut gi = game_item.clone();
            area.signals()
                .area_entered()
                .connect(move |area| gi.bind_mut().on_area_entered(area));
            let mut gi = game_item.clone();
            area.signals()
                .area_exited()
                .connect(move |area| gi.bind_mut().on_area_exited(area));
        }
    }

    fn fade_camera_out(&mut self) -> godot::task::SignalFuture<()> {
        let mut player = self.base().get_node_as::<MainCharacter>(
            GlobalData::singleton()
                .bind()
                .paths
                .player
                .as_ref()
                .unwrap(),
        );
        player.bind_mut().force_disabled();

        let mut tree = self.base().get_tree().unwrap();
        let mut layer = CanvasLayer::new_alloc();
        let mut rect = ColorRect::new_alloc();

        rect.set_anchor(Side::RIGHT, 1.0);
        rect.set_anchor(Side::BOTTOM, 1.0);
        rect.set_modulate(Color::from_rgba(0.0, 0.0, 0.0, 0.0));
        rect.set_name("tween_rect");
        layer.add_child(&rect);
        layer.set_name("tween_layer");
        self.base_mut().add_child(&layer);

        let mut tween = tree.create_tween().unwrap();
        tween.tween_property(&rect, "modulate:a", &1.0.to_variant(), 0.5);
        tween.signals().finished().to_future()
    }

    fn fade_camera_in(&mut self) {
        let mut this = self.to_gd();
        let mut tree = self.base().get_tree().unwrap();
        let mut player = self.base().get_node_as::<MainCharacter>(
            GlobalData::singleton()
                .bind()
                .paths
                .player
                .as_ref()
                .unwrap(),
        );
        player.bind_mut().force_enabled();

        let layer = self.base().get_node_as::<CanvasLayer>("tween_layer");
        let rect = layer.get_node_as::<ColorRect>("tween_rect");

        godot::task::spawn(async move {
            let mut tween = tree.create_tween().unwrap();
            tween.tween_property(&rect, "modulate:a", &0.0.to_variant(), 0.7);
            tween.signals().finished().to_future().await;
            this.apply_deferred(move |this| this.base_mut().remove_child(&layer));
        });
    }

    fn on_player_camera_request_detach(&mut self, pos: Vector2) {
        if let Some(path) = &GlobalData::singleton().bind().paths.player {
            let mut player = self.base().get_node_as::<MainCharacter>(path);

            if let Some(mut p_cam) = player.try_get_node_as::<PlayerCamera>("ShakyPlayerCamera") {
                println!("Removing camera");
                player.remove_child(&p_cam);
                self.base_mut().add_child(&p_cam);
                p_cam.set_global_position(pos);
                self.temp_holder = Some(p_cam);
                println!("Main temp holder: {:?}", self.temp_holder);
                println!(
                    "Main temp holder path: {:?}",
                    self.temp_holder.as_ref().unwrap().get_path()
                );
            } else {
                println!("No camera");
            }
        }
    }

    fn on_player_camera_request_attach(&mut self) {
        println!("Attaching camera");
        if let Some(path) = &GlobalData::singleton().bind().paths.player {
            let mut player = self.base().get_node_as::<MainCharacter>(path);
            self.temp_holder
                .as_mut()
                .unwrap()
                .set_position(Vector2::new(0.0, 0.0));
            let p_cam = self.temp_holder.take().unwrap();
            self.base_mut().remove_child(&p_cam);
            player.add_child(self.temp_holder.as_ref().unwrap());
        }
    }
}
