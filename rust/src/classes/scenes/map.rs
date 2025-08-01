use godot::{
    classes::{Marker2D, NavigationRegion2D, TileMapLayer},
    prelude::*,
};

use crate::{
    classes::scenes::environment_trigger::{EnvironmentTrigger, MapTransition},
    components::managers::item::GameItem,
    utils::constants,
};

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct Map {
    pub player_spawn_pos: Vector2,
    pub map_layers: Vec<Gd<TileMapLayer>>,
    pub triggers: Vec<Gd<EnvironmentTrigger>>,
    pub nav_regions: Vec<Gd<NavigationRegion2D>>,
    pub items: Vec<Gd<GameItem>>,

    base: Base<Node>,
}

#[godot_api]
impl INode for Map {
    fn enter_tree(&mut self) {
        constants::get_world_data()
            .bind_mut()
            .paths
            .map
            .replace(self.base().get_path().to_string());
    }

    fn exit_tree(&mut self) {
        constants::get_world_data().bind_mut().paths.map.take();
    }

    fn ready(&mut self) {
        self.player_spawn_pos = self
            .base()
            .get_node_as::<Marker2D>("PlayerSpawnPos")
            .get_global_position();

        let map_trans = self
            .base()
            .try_get_node_as::<MapTransition>("Environment/MapTransition");
        if let Some(map) = map_trans {
            map.signals()
                .transition_maps()
                .connect_other(&self.to_gd(), Self::on_map_transition_req);
        }

        let layers = self
            .base()
            .get_node_as::<Node>("TileMapLayers")
            .get_children();
        self.map_layers = layers
            .iter_shared()
            .map(|n| n.cast::<TileMapLayer>())
            .collect();

        let triggers = self
            .base()
            .get_node_as::<Node>("Environment/EnvironmentTriggers")
            .get_children();

        self.triggers = triggers
            .iter_shared()
            .map(|n| n.cast::<EnvironmentTrigger>())
            .collect();

        let nav_regions = self.base().get_node_as::<Node>("NavRegions").get_children();
        self.nav_regions = nav_regions
            .iter_shared()
            .map(|n| n.cast::<NavigationRegion2D>())
            .collect();

        let items = self.base().get_node_as::<Node>("Items").get_children();
        self.items = items.iter_shared().map(|n| n.cast::<GameItem>()).collect();
    }
}

#[godot_api]
impl Map {
    #[signal]
    pub fn propigate_map_trans(next_map: Gd<PackedScene>);

    fn on_map_transition_req(&mut self, next_map: Gd<PackedScene>) {
        self.signals().propigate_map_trans().emit(&next_map);
    }
}
